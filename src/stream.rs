use tokio::time;
use std::time::Duration;
use web3::futures::{self, StreamExt};

struct Monitor {
    counter: u64,
}

impl Monitor {
    fn inc(mut self) -> Self {
        self.counter +=1;
        self
    }
}

const A_SECOND: Duration = Duration::from_secs(1);

async fn get_events(m: &Monitor) -> u64 {
    time::sleep(A_SECOND).await;
    m.counter
}


// See https://stackoverflow.com/questions/58700741/is-there-any-way-to-create-a-async-stream-generator-that-yields-the-result-of-re
fn get_event_stream() -> impl futures::Stream<Item = u64> {
    futures::stream::unfold( Monitor {counter: 1}, |mon| async {
        let events = get_events(&mon).await;
        Some((events, mon.inc()))
    })
}

async fn test() {
    get_event_stream()
    .take(3)
    .for_each(|v| async move {
        println!("Got the id {}", v);
    })
    .await;

    let mut x = Box::pin(get_event_stream());
    while let Some(data) = x.next().await {
        dbg!(data);
    }
}
