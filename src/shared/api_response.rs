use serde::Serialize;

#[allow(dead_code)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub success: bool,
    pub message: String,
    pub data: T,
}
