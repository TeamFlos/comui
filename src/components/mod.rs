pub mod button;
pub mod label;

pub trait DataComponent<D> {
    fn set_data(&mut self, data: D);
    fn get_data(&self) -> &D;
}
