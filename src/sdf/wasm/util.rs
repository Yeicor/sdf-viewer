pub fn reinterpret_u32_as_i32(sdf_id: u32) -> i32 {
    i32::from_le_bytes(sdf_id.to_le_bytes()) // Reinterpret without modifications
}

pub fn reinterpret_i32_as_u32(sdf_id: i32) -> u32 {
    u32::from_le_bytes(sdf_id.to_le_bytes()) // Reinterpret without modifications
}
