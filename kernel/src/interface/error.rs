use error_stack::Report;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Delegate(Box<dyn std::error::Error + Sync + Send>);

impl<C> From<Report<C>> for Delegate 
where 
    C: 'static
{
    fn from(value: Report<C>) -> Self {
        Delegate(Box::new(value.into_error()))
    }
}