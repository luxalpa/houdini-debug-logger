use std::net::{Ipv4Addr, SocketAddrV4};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use crate::loggable::DebugLoggable;
use anyhow::{anyhow, Result};
use hapi_rs::attribute::{AttributeInfo, StorageType};
use hapi_rs::enums::{AttributeOwner, AttributeTypeInfo, PartType};
use hapi_rs::geometry::PartInfo;
use hapi_rs::node::{Geometry, HoudiniNode};
use hapi_rs::session::{connect_to_socket, quick_session, Session};

/// Trait that can be implemented for converting any types into a loggable type. Theoretically,
/// DebugLoggable could be used instead, but that would require making the HDA aware of the new type.
/// Using this one instead is typically preferred and mostly just a convenience helper to be able
/// to pass custom types directly into [`houlog`].
pub trait IntoLoggable {
    /// The loggable type that this type can be converted into.
    type LoggableType: DebugLoggable + 'static;

    /// Convert the type into a loggable type.
    fn into_loggable(self) -> Self::LoggableType;
}

impl<T: DebugLoggable + 'static> IntoLoggable for T {
    type LoggableType = T;
    fn into_loggable(self) -> Self::LoggableType {
        self
    }
}

/// The main logging function. Please note that this currently operates on global state.
pub fn houlog<T: IntoLoggable>(name: &str, v: T) {
    let logger = match HOUDINI_DEBUG_LOGGER.get() {
        Some(logger) => logger,
        None => {
            log::warn!("HoudiniDebugLogger not initialized");
            return;
        }
    };
    logger.log(name, v.into_loggable()).unwrap();
}

/// Advance the logger to the next frame. When first initializing the logger, it starts on frame 0,
/// so typically this is only needed when you want to log data for multiple frames.
/// This is the frames in the recording, it does not have to be actual frames in your code. For
/// example, a world generation algorithm could separate the different stages of the generation into
/// different frames.
pub fn houlog_next_frame() -> Result<()> {
    let logger = match HOUDINI_DEBUG_LOGGER.get() {
        Some(logger) => logger,
        None => {
            log::warn!("HoudiniDebugLogger not initialized");
            return Ok(());
        }
    };
    logger.next_frame()
}

/// This initializes houlog to write to a file. Typically, you'd want to use [`init_houlog_live`]
/// instead which gives immediate feedback without needing to manually reload.
pub fn init_houlog(path: impl Into<PathBuf>) -> Result<()> {
    HOUDINI_DEBUG_LOGGER
        .set(HoudiniDebugLogger::new_with_file(path.into()))
        .map_err(|_| anyhow!("HoudiniDebugLogger already initialized"))
}

/// This initializes houlog to write to a live Houdini session. If you're already attached to a
/// session for a different purpose (for example live-reloading), you can pass it in here.
/// You must have a live session running in Houdini which you can start via the
/// "Houdini Engine SessionSync" pane tab (which can be found clicking on the + and then under New Pane Tab Type -> Misc).
pub fn init_houlog_live(session: Option<Session>) -> Result<()> {
    HOUDINI_DEBUG_LOGGER
        .set(HoudiniDebugLogger::new_with_live_session(session)?)
        .map_err(|_| anyhow!("HoudiniDebugLogger already initialized"))
}

/// Save the session and send it to Houdini.
pub fn save_houlog() -> Result<()> {
    let logger = match HOUDINI_DEBUG_LOGGER.get() {
        Some(logger) => logger,
        None => {
            log::warn!("HoudiniDebugLogger not initialized");
            return Ok(());
        }
    };
    logger.save()
}

static HOUDINI_DEBUG_LOGGER: OnceLock<HoudiniDebugLogger> = OnceLock::new();

/// The method of exporting the data. This can either be a live session or a file.
pub enum ExportMethod {
    LiveSession {
        /// The hapi-rs session to use.
        session: Session,

        /// The path to the subnet in which the node will be stored
        path: String,

        /// The name of the node
        node_name: String,
    },
    File {
        /// The full filepath to the file to be created. Typically, this should end with `.bgeo`.
        path: PathBuf,
    },
}

struct LogEntry {
    name: String,
    value: Box<dyn DebugLoggable>,
}

struct FrameData {
    entries: Vec<LogEntry>,
}

impl FrameData {
    fn new() -> Self {
        FrameData {
            entries: Vec::new(),
        }
    }
}

struct LoggerData {
    modified: bool,
    frames: Vec<FrameData>,
}

struct HoudiniDebugLogger {
    data: Mutex<LoggerData>,
    export_method: ExportMethod,
}

impl HoudiniDebugLogger {
    fn new_with_file(p: PathBuf) -> Self {
        HoudiniDebugLogger {
            export_method: ExportMethod::File { path: p },
            data: Mutex::new(LoggerData {
                modified: true,
                frames: vec![FrameData::new()],
            }),
        }
    }

    fn new_with_live_session(session: Option<Session>) -> Result<Self> {
        let session = match session {
            Some(session) => session,
            None => {
                let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 9090);
                connect_to_socket(socket, None)?
            }
        };

        Ok(HoudiniDebugLogger {
            export_method: ExportMethod::LiveSession {
                session,
                path: "/obj/recordings".to_string(),
                node_name: "recording".to_string(),
            },
            data: Mutex::new(LoggerData {
                modified: true,
                frames: vec![FrameData::new()],
            }),
        })
    }

    fn next_frame(&self) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| anyhow!("error during lock"))?;
        data.modified = true;
        data.frames.push(FrameData::new());
        Ok(())
    }

    fn log<T: DebugLoggable + 'static>(&self, name: &str, v: T) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| anyhow!("error during lock"))?;
        data.modified = true;
        let frame_data = data
            .frames
            .last_mut()
            .ok_or_else(|| anyhow!("For some reason no active frame was found"))?;
        frame_data.entries.push(LogEntry {
            name: name.to_string(),
            value: Box::new(v),
        });
        Ok(())
    }

    fn save(&self) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| anyhow!("error during lock"))?;
        if !data.modified {
            // Avoid saving overly often
            return Ok(());
        }
        data.modified = false;

        let node = Self::create_output_node(&self.export_method)?;
        node.cook()?;
        let geom = node
            .geometry()?
            .ok_or_else(|| anyhow!("No geometry on node"))?;

        let num_points = data
            .frames
            .iter()
            .map(|frame| frame.entries.len())
            .sum::<usize>();

        let part_info = PartInfo::default()
            .with_part_type(PartType::Mesh)
            .with_point_count(num_points as i32);

        geom.set_part_info(&part_info)?;

        Self::add_positions(&geom, &data.frames)?;
        Self::add_names(&geom, &data.frames)?;
        Self::add_frame_times(&geom, &data.frames)?;
        Self::add_metadata(&geom, &data.frames)?;
        Self::add_kinds(&geom, &data.frames)?;

        geom.commit()?;

        if let ExportMethod::File { path } = &self.export_method {
            geom.save_to_file(
                path.to_str()
                    .ok_or_else(|| anyhow!("Could not convert path to string"))?,
            )?;
        }

        Ok(())
    }

    fn add_positions(geom: &Geometry, frames: &[FrameData]) -> Result<()> {
        let point_positions = frames
            .iter()
            .flat_map(|frame| frame.entries.iter().map(|entry| entry.value.position()))
            .flat_map(|v| vec![v.x, v.y, v.z])
            .collect::<Vec<f32>>();

        let p_attr_info = AttributeInfo::default()
            .with_count(point_positions.len() as i32 / 3)
            .with_tuple_size(3)
            .with_storage(StorageType::Float)
            .with_type_info(AttributeTypeInfo::Point)
            .with_owner(AttributeOwner::Point);

        let p_attrib = geom.add_numeric_attribute::<f32>("P", 0, p_attr_info)?;

        if !point_positions.is_empty() {
            p_attrib.set(0, &point_positions)?;
        }

        Ok(())
    }

    fn add_names(geom: &Geometry, frames: &[FrameData]) -> Result<()> {
        let point_names = frames
            .iter()
            .flat_map(|frame| frame.entries.iter().map(|entry| entry.name.clone()))
            .collect::<Vec<String>>();

        let name_attr_info = AttributeInfo::default()
            .with_count(point_names.len() as i32)
            .with_tuple_size(1)
            .with_storage(StorageType::String)
            .with_owner(AttributeOwner::Point);

        let name_attrib = geom.add_string_attribute("name", 0, name_attr_info)?;

        if !point_names.is_empty() {
            name_attrib.set(
                0,
                point_names
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )?;
        }

        Ok(())
    }

    fn add_kinds(geom: &Geometry, frames: &[FrameData]) -> Result<()> {
        let point_kinds = frames
            .iter()
            .flat_map(|frame| frame.entries.iter().map(|entry| entry.value.kind().clone()))
            .collect::<Vec<String>>();

        let kind_attr_info = AttributeInfo::default()
            .with_count(point_kinds.len() as i32)
            .with_tuple_size(1)
            .with_storage(StorageType::String)
            .with_owner(AttributeOwner::Point);

        let kind_attrib = geom.add_string_attribute("kind", 0, kind_attr_info)?;

        if !point_kinds.is_empty() {
            kind_attrib.set(
                0,
                point_kinds
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )?;
        }

        Ok(())
    }

    fn add_frame_times(geom: &Geometry, frames: &[FrameData]) -> Result<()> {
        let point_times = frames
            .iter()
            .enumerate()
            .flat_map(|(frame, d)| d.entries.iter().map(move |_| (frame + 1) as f32))
            .collect::<Vec<f32>>();

        let time_attr_info = AttributeInfo::default()
            .with_count(point_times.len() as i32)
            .with_tuple_size(1)
            .with_storage(StorageType::Float)
            .with_owner(AttributeOwner::Point);

        let time_attrib = geom.add_numeric_attribute::<f32>("time", 0, time_attr_info)?;

        if !point_times.is_empty() {
            time_attrib.set(0, point_times.as_slice())?;
        }

        Ok(())
    }

    fn add_metadata(geom: &Geometry, frames: &[FrameData]) -> Result<()> {
        let pt_metadata = frames
            .iter()
            .flat_map(|frame| frame.entries.iter().map(|entry| entry.value.as_json()))
            .collect::<Vec<String>>();

        let metadata_attr_info = AttributeInfo::default()
            .with_count(pt_metadata.len() as i32)
            .with_tuple_size(1)
            .with_storage(StorageType::String)
            .with_owner(AttributeOwner::Point);

        let name_attrib = geom.add_string_attribute("metadata", 0, metadata_attr_info)?;

        if !pt_metadata.is_empty() {
            name_attrib.set(
                0,
                pt_metadata
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    .as_slice(),
            )?;
        }

        Ok(())
    }

    fn create_output_node(export_method: &ExportMethod) -> Result<HoudiniNode> {
        let node = match export_method {
            ExportMethod::LiveSession {
                session,
                path,
                node_name,
            } => {
                let parent = session.get_node_from_path(path, None)?.unwrap();
                if let Some(handle) = session.get_node_from_path(node_name, Some(parent.handle))? {
                    session.delete_node(handle)?;
                }
                session
                    .node_builder("null")
                    .with_parent(parent)
                    .with_label(node_name)
                    .create()?
            }
            ExportMethod::File { .. } => {
                let session = quick_session(None)?;
                let parent = session.create_node("Object/geo")?;
                session.node_builder("null").with_parent(parent).create()?
            }
        };
        Ok(node)
    }
}

impl Drop for HoudiniDebugLogger {
    fn drop(&mut self) {
        self.save().unwrap_or_else(|e| {
            println!("Failed to save Houdini Debug Log: {}", e);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Armature, Capsule, Line, Polygon, Polyline};
    use glam::{Mat4, Quat, Vec3};

    #[test]
    fn test() -> Result<()> {
        // init_houlog("./houlog.bgeo")?;
        init_houlog_live(None)?;
        houlog("test", Vec3::new(1.0, 2.0, 3.0));
        houlog(
            "test",
            Mat4::from_rotation_translation(
                Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()),
                Vec3::new(5.0, 0.0, 0.0),
            ),
        );
        houlog(
            "test-line",
            Line {
                start: Vec3::new(0.5, 0.5, 0.0),
                end: Vec3::new(1.0, 1.0, 1.0),
            },
        );

        houlog(
            "test-poly-line",
            Polyline {
                points: vec![
                    Vec3::new(2.5, 2.5, 0.0),
                    Vec3::new(2.0, 2.0, 2.0),
                    Vec3::new(2.0, 3.0, 2.0),
                ],
            },
        );

        houlog(
            "test-poly",
            Polygon {
                points: vec![
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(1.0, 1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0),
                ],
            },
        );

        houlog(
            "test-armature",
            Armature {
                names: vec!["root".to_string(), "child".to_string()],
                parents: vec![-1, 0],
                xforms: vec![
                    Mat4::IDENTITY,
                    Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0)),
                ],
            },
        );

        houlog(
            "test-capsule",
            Capsule {
                point_a: Vec3::new(0.0, 0.0, 0.0),
                point_b: Vec3::new(1.0, 0.0, 0.0),
                radius: 0.5,
            },
        );

        save_houlog()?;

        Ok(())
    }
}
