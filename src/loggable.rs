use crate::IntoLoggable;
use glam::{Mat4, Quat, Vec3};
use serde_json::json;

/// A trait for types that can be logged to Houdini. This must be kept in sync with the HDA or
/// houdini node that parses the log data. For just logging a custom type, use the [`IntoLoggable`]
/// trait if possible.
pub trait DebugLoggable: Send {
    /// The kind of the data, for example `mat4` or `vec3`.
    fn kind(&self) -> String;

    /// The "root" position of the data. Not all data types necessarily have a meaningful position,
    /// which is fine.
    fn position(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    /// The metadata of the data, as a JSON string.
    fn as_json(&self) -> String;
}

impl DebugLoggable for Vec3 {
    fn kind(&self) -> String {
        "vec3".to_string()
    }

    fn position(&self) -> Vec3 {
        *self
    }

    fn as_json(&self) -> String {
        json!(
            {
                "pt": [self.x, self.y, self.z]
            }
        )
        .to_string()
    }
}

impl DebugLoggable for Mat4 {
    fn kind(&self) -> String {
        "mat4".to_string()
    }

    fn position(&self) -> Vec3 {
        self.w_axis.truncate()
    }

    fn as_json(&self) -> String {
        json!(
            {
                "xform": [
                    self.x_axis.x, self.x_axis.y, self.x_axis.z, self.x_axis.w,
                    self.y_axis.x, self.y_axis.y, self.y_axis.z, self.y_axis.w,
                    self.z_axis.x, self.z_axis.y, self.z_axis.z, self.z_axis.w,
                    self.w_axis.x, self.w_axis.y, self.w_axis.z, self.w_axis.w,
                ]
            }
        )
        .to_string()
    }
}

impl DebugLoggable for Quat {
    fn kind(&self) -> String {
        "quat".to_string()
    }

    fn position(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    fn as_json(&self) -> String {
        json!(
            {
                "quat": [self.x, self.y, self.z, self.w]
            }
        )
        .to_string()
    }
}

impl DebugLoggable for f32 {
    fn kind(&self) -> String {
        "float".to_string()
    }

    fn position(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }

    fn as_json(&self) -> String {
        json!({ "float": self }).to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Polyline {
    pub points: Vec<Vec3>,
}

impl DebugLoggable for Polyline {
    fn kind(&self) -> String {
        "line".to_string()
    }

    fn position(&self) -> Vec3 {
        self.points[0]
    }

    fn as_json(&self) -> String {
        let x = self.points.iter().map(|pt| pt.x).collect::<Vec<f32>>();
        let y = self.points.iter().map(|pt| pt.y).collect::<Vec<f32>>();
        let z = self.points.iter().map(|pt| pt.z).collect::<Vec<f32>>();

        json!({
            "x": x,
            "y": y,
            "z": z,
        })
        .to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

impl IntoLoggable for Line {
    type LoggableType = Polyline;
    fn into_loggable(self) -> Self::LoggableType {
        Polyline {
            points: vec![self.start, self.end],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub points: Vec<Vec3>,
}

impl DebugLoggable for Polygon {
    fn kind(&self) -> String {
        "polygon".to_string()
    }

    fn position(&self) -> Vec3 {
        self.points[0]
    }

    fn as_json(&self) -> String {
        let x = self.points.iter().map(|pt| pt.x).collect::<Vec<f32>>();
        let y = self.points.iter().map(|pt| pt.y).collect::<Vec<f32>>();
        let z = self.points.iter().map(|pt| pt.z).collect::<Vec<f32>>();

        json!({
            "x": x,
            "y": y,
            "z": z,
        })
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<usize>,
    pub index_counts: Vec<usize>,
}

impl DebugLoggable for Mesh {
    fn kind(&self) -> String {
        "mesh".to_string()
    }

    fn position(&self) -> Vec3 {
        self.vertices[0]
    }

    fn as_json(&self) -> String {
        let x = self.vertices.iter().map(|pt| pt.x).collect::<Vec<f32>>();
        let y = self.vertices.iter().map(|pt| pt.y).collect::<Vec<f32>>();
        let z = self.vertices.iter().map(|pt| pt.z).collect::<Vec<f32>>();

        json!({
            "x": x,
            "y": y,
            "z": z,
            "i": self.indices,
            "c": self.index_counts,
        })
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Armature {
    pub names: Vec<String>,
    pub parents: Vec<i32>,
    pub xforms: Vec<Mat4>,
}

impl DebugLoggable for Armature {
    fn kind(&self) -> String {
        "armature".to_string()
    }

    fn position(&self) -> Vec3 {
        self.xforms[0].w_axis.truncate()
    }

    fn as_json(&self) -> String {
        let xforms = self
            .xforms
            .iter()
            .flat_map(|xform| xform.to_cols_array())
            .collect::<Vec<f32>>();

        json!({
            "names": self.names,
            "xforms": xforms,
            "parents": self.parents,
        })
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Capsule {
    pub point_a: Vec3,
    pub point_b: Vec3,
    pub radius: f32,
}

impl DebugLoggable for Capsule {
    fn kind(&self) -> String {
        "capsule".to_string()
    }

    fn position(&self) -> Vec3 {
        (self.point_a + self.point_b) / 2.0
    }

    fn as_json(&self) -> String {
        json!({
            "a": [self.point_a.x, self.point_a.y, self.point_a.z],
            "b": [self.point_b.x, self.point_b.y, self.point_b.z],
            "r": self.radius,
        })
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}

impl DebugLoggable for Sphere {
    fn kind(&self) -> String {
        "sphere".to_string()
    }

    fn position(&self) -> Vec3 {
        self.center
    }

    fn as_json(&self) -> String {
        json!({
            "radius": self.radius,
        })
        .to_string()
    }
}
