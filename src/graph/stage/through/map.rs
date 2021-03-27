use crate::graph::shape::{Shape, SinkShape, SourceShape, ThroughShape};
use crate::graph::{GraphResult, Inlet, Outlet, Port, Stage};
use crate::AppData;
use async_trait::async_trait;
use cast_trait_object::dyn_upcast;
use std::fmt;

/// Transform this stream by applying the given function to each of the elements as they pass
/// through this processing step.
///
/// # Examples
///
/// ```
/// use tokio::sync::mpsc;
/// use proctor::graph::{Connect, Inlet};
/// use proctor::graph::stage::{self, Stage};
/// use proctor::graph::{ThroughShape, SinkShape, SourceShape};
///
/// #[tokio::main]
/// async fn main() {
///     let my_data = vec![1, 2, 3];
///     let (tx, rx) = mpsc::channel(8);
///
///     let mut sq = stage::Map::new("square values", |x| x * x);
///     let mut fold = stage::Fold::new("sum values", 0, |acc, x| acc + x );
///     let mut rx_sum_sq = fold.take_final_rx().unwrap();
///
///     sq.inlet().attach(rx).await;
///     (sq.outlet(), fold.inlet()).connect().await;
///
///     let sq_handle = tokio::spawn(async move { sq.run().await; });
///     let fold_handle = tokio::spawn(async move { fold.run().await; });
///     let source_handle = tokio::spawn(async move {
///         for x in my_data { tx.send(x).await.expect("failed to send data"); }
///     });
///
///     source_handle.await.unwrap();
///     sq_handle.await.unwrap();
///     fold_handle.await.unwrap();
///
///     match rx_sum_sq.try_recv() {
///         Ok(sum_sq) => assert_eq!(14, sum_sq),
///         Err(err) => panic!("sum of squares not calculated: {}", err),
///     }
/// }
/// ```
pub struct Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
    name: String,
    operation: F,
    inlet: Inlet<In>,
    outlet: Outlet<Out>,
}

impl<F, In, Out> Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
    pub fn new<S: Into<String>>(name: S, operation: F) -> Self {
        let name = name.into();
        let inlet = Inlet::new(name.clone());
        let outlet = Outlet::new(name.clone());
        Self {
            name,
            operation,
            inlet,
            outlet,
        }
    }
}

impl<F, In, Out> Shape for Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
}

impl<F, In, Out> ThroughShape for Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
}

impl<F, In, Out> SourceShape for Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
    type Out = Out;
    #[inline]
    fn outlet(&mut self) -> &mut Outlet<Self::Out> {
        &mut self.outlet
    }
}

impl<F, In, Out> SinkShape for Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
    type In = In;
    #[inline]
    fn inlet(&mut self) -> &mut Inlet<Self::In> {
        &mut self.inlet
    }
}

#[dyn_upcast]
#[async_trait]
impl<F, In, Out> Stage for Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send + 'static,
    In: AppData,
    Out: AppData,
{
    #[inline]
    fn name(&self) -> &str {
        self.name.as_str()
    }

    #[tracing::instrument(
        level="info",
        name="run map through",
        skip(self),
        fields(name=%self.name),
    )]
    async fn run(&mut self) -> GraphResult<()> {
        let outlet = &self.outlet;
        // let op = &self.operation;
        while let Some(input) = self.inlet.recv().await {
            let value = (self.operation)(input);
            outlet.send(value).await?;
        }

        Ok(())
    }

    async fn close(mut self: Box<Self>) -> GraphResult<()> {
        tracing::trace!("closing map-through ports.");
        self.inlet.close().await;
        self.outlet.close().await;
        Ok(())
    }
}

impl<F, In, Out> fmt::Debug for Map<F, In, Out>
where
    F: FnMut(In) -> Out + Send,
    In: AppData,
    Out: AppData,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Map")
            .field("name", &self.name)
            .field("inlet", &self.inlet)
            .field("outlet", &self.outlet)
            .finish()
    }
}

/////////////////////////////////////////////////////
// Unit Tests ///////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio_test::block_on;

    #[test]
    fn test_basic_usage() {
        let my_data = vec![1, 2, 3];
        let (tx_in, rx_in) = mpsc::channel(8);
        let (tx_out, mut rx_out) = mpsc::channel(8);

        let mut map = Map::new("square values", |x| x * x);

        let mut actual = Vec::with_capacity(3);

        block_on(async {
            map.inlet.attach(rx_in).await;
            map.outlet.attach(tx_out).await;

            let map_handle = tokio::spawn(async move {
                map.run().await.expect("failed on map run");
            });

            let source_handle = tokio::spawn(async move {
                for x in my_data {
                    tx_in.send(x).await.expect("failed to send data");
                }
            });

            source_handle.await.unwrap();
            map_handle.await.unwrap();
            while let Some(x) = rx_out.recv().await {
                actual.push(x);
            }
        });

        assert_eq!(vec![1, 4, 9], actual);
    }
}
