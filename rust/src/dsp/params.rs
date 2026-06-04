use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseGateParams   { pub threshold: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressorParams  { pub threshold_db: f32, pub ratio: f32, pub attack: f32, pub release: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverdriveParams   { pub drive: f32, pub tone: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistortionParams  { pub drive: f32, pub level: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzParams        { pub fuzz: f32, pub level: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChorusParams      { pub rate: f32, pub depth: f32, pub mix: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TremoloParams     { pub rate: f32, pub depth: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelayParams       { pub time_ms: f32, pub feedback: f32, pub mix: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverbParams      { pub room_size: f32, pub mix: f32 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoostParams       { pub gain: f32 }
