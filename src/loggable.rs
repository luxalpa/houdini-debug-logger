use crate::IntoLoggable;
use glam::{Mat4, Quat, Vec3};
use serde_json::json;

pub trait DebugLoggable: Send {
    fn kind(&self) -> String;
    fn position(&self) -> Vec3 {
        Vec3::new(0.0, 0.0, 0.0)
    }
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
