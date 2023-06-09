use super::{arc_work_model};

impl arc_work_model::Output {
    pub fn update_image_meta(&mut self) -> anyhow::Result<()> {
        self.image_meta.analyze(&self.image)?;
        Ok(())
    }
}
