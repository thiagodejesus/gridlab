use grid_engine::grid_engine::GridEngine;

pub struct GridDeliver {
    pub grid: GridEngine,
}

impl TryFrom<Vec<u8>> for GridDeliver {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let grid =
            GridEngine::try_from(&value).map_err(|_| "Failed to convert Vec<u8> to GridEngine")?;
        Ok(Self { grid })
    }
}
