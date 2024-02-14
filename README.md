# Log 3D data into Houdini

Tool for logging per frame debugging data to a Houdini Live Session for inspection and debugging. For example, in order to visualize the effects of matrix transformations, or be able to step through a more complicated 3D algorithm such as a world generator.

Simply log the geometric data, then step through the recording frame by frame in Houdini in order to inspect the individual values.

This outputs the metadata as JSON data on a Node, which then can be parsed as a dict using Vex inside Houdini.

## Installation

- The hapi-rs dependency used here requires `HFS` environment variable to be set, for example via config.toml.

## Usage

```rust
fn main() -> Result<()> {
    // initialize via Live Session. Per default the node that's created will be in /obj/recordings subnet with the name "recording"
    init_houlog_live(None)?;
    
    // Log a Vec3
    houlog("test", Vec3::new(1.0, 2.0, 3.0));

    // Move to the next frame
    houlog_next_frame()?;
    
    // Log a Mat4
    houlog(
        "test",
        Mat4::from_rotation_translation(
            Quat::from_axis_angle(Vec3::Y, 45f32.to_radians()),
            Vec3::new(5.0, 0.0, 0.0),
        ),
    );
    
    // Log a Line
    houlog(
        "test-line",
        Line {
            start: Vec3::new(0.5, 0.5, 0.0),
            end: Vec3::new(1.0, 1.0, 1.0),
        },
    );

    // Log a Polyline
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

    // Log a Polygon
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

    // Serialize the data and then send it to Houdini
    save_houlog()?;
}
```

For custom geometry types, the `IntoLoggable` trait can be implemented:

```rust
pub struct Line2D {
    pub start: Vec2,
    pub end: Vec2,
}

impl houdini_debug_logger::IntoLoggable for Line2D {
    type LoggableType = houdini_debug_logger::Polyline;
    fn into_loggable(self) -> Self::LoggableType {
        houdini_debug_logger::Line {
            start: glam::Vec3::new(self.start.x, 0.0, self.start.y),
            end: glam::Vec3::new(self.end.x, 0.0, self.end.y),
        }
        .into_loggable()
    }
}
```

