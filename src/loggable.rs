use glam::{Mat4, Quat, Vec3};
use serde_json::json;

pub trait DebugLoggable: Send {
    fn position(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    fn as_json(&self) -> String;
    fn kind(&self) -> String;
}

impl DebugLoggable for Vec3 {
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

    fn kind(&self) -> String {
        "vec3".to_string()
    }
}

impl DebugLoggable for Mat4 {
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

    fn kind(&self) -> String {
        "mat4".to_string()
    }
}

impl DebugLoggable for Quat {
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

    fn kind(&self) -> String {
        "quat".to_string()
    }
}

impl DebugLoggable for f32 {
    fn position(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
    fn as_json(&self) -> String {
        json!({ "float": self }).to_string()
    }

    fn kind(&self) -> String {
        "float".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Line {
    pub start: Vec3,
    pub end: Vec3,
}

impl DebugLoggable for Line {
    fn position(&self) -> Vec3 {
        self.start
    }
    fn as_json(&self) -> String {
        json!(
            {
                "pt1": [
                    self.start.x, self.start.y, self.start.z,
                ],
                "pt2": [
                    self.end.x, self.end.y, self.end.z,
                ],
            }
        )
        .to_string()
    }

    fn kind(&self) -> String {
        "line".to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Polygon {
    pub points: Vec<Vec3>,
}

impl DebugLoggable for Polygon {
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

    fn kind(&self) -> String {
        "polygon".to_string()
    }
}
