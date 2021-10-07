use tracing::debug;
use html_parser::{Dom, Node};
use url::Url;
use std::sync::Mutex as StdMutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

type OpaqueError = Box<dyn std::error::Error + Send + Sync>;
type Res<T> = Result<T, OpaqueError>;

const WIKIPEDIA: &'static str = "http://en.wikipedia.org/";

const SITES: &[&'static str] = &[
    WIKIPEDIA
];

static HANDLE_COUNT: AtomicUsize = AtomicUsize::new(0);
static RETRY_COUNT: AtomicUsize = AtomicUsize::new(0);
static ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);

static ATTEMPT_COUNT: AtomicUsize = AtomicUsize::new(0);
static RESPONSE_COUNT: AtomicUsize = AtomicUsize::new(0);
static DECODED_COUNT: AtomicUsize = AtomicUsize::new(0);
static PROCESSED_COUNT: AtomicUsize = AtomicUsize::new(0);
static BAILOUT_COUNT: AtomicUsize = AtomicUsize::new(0);

const MAX_DEPTH: u32 = 3;

#[tokio::main(worker_threads = 2)]
async fn main() -> Res<()> {
    println!("Hello, world!");

    std::thread::spawn(|| {
        let mut last = Instant::now();
        let mut last_attempt = ATTEMPT_COUNT.load(Ordering::SeqCst);
        let mut last_response = RESPONSE_COUNT.load(Ordering::SeqCst);
        let mut last_decoded = DECODED_COUNT.load(Ordering::SeqCst);
        let mut last_processed = PROCESSED_COUNT.load(Ordering::SeqCst);
        let mut last_bailouts = BAILOUT_COUNT.load(Ordering::SeqCst);
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let now = Instant::now();
            let now_attempt = ATTEMPT_COUNT.load(Ordering::SeqCst);
            let now_response = RESPONSE_COUNT.load(Ordering::SeqCst);
            let now_decoded = DECODED_COUNT.load(Ordering::SeqCst);
            let now_processed = PROCESSED_COUNT.load(Ordering::SeqCst);
            let now_bailouts = BAILOUT_COUNT.load(Ordering::SeqCst);
            let delta_attempt = now_attempt - last_attempt;
            let delta_response = now_response - last_response;
            let delta_decoded = now_decoded - last_decoded;
            let delta_processed = now_processed - last_processed;
            let delta_bailouts = now_bailouts - last_bailouts;
            let delta_time = now - last;
            let attempt_per_sec = (delta_attempt * 1000) as u128 /  delta_time.as_millis();
            let response_per_sec = (delta_response * 1000) as u128 / delta_time.as_millis();
            let decoded_per_sec = (delta_decoded * 1000) as u128 / delta_time.as_millis();
            let processed_per_sec = (delta_processed * 1000) as u128 / delta_time.as_millis();
            let bailouts_per_sec = (delta_bailouts * 1000) as u128 / delta_time.as_millis();
            if false {
                println!("handles: {} errors: {} retries: {} \
                          attempts: {} ({}/sec) \
                          responses: {} ({}/sec) \
                          decodes: {} ({}/sec) \
                          processed: {} ({}/sec) \
                          bailouts: {} ({}/sec)",
                         HANDLE_COUNT.load(Ordering::SeqCst),
                         ERROR_COUNT.load(Ordering::SeqCst),
                         RETRY_COUNT.load(Ordering::SeqCst),
                         now_attempt, attempt_per_sec,
                         now_response, response_per_sec,
                         now_decoded, decoded_per_sec,
                         now_processed, processed_per_sec,
                         now_bailouts, bailouts_per_sec,
                );
            }
            last_attempt = now_attempt;
            last_processed = now_processed;
            last_decoded = now_decoded;
            last_response = now_response;
            last_bailouts = now_bailouts;
        }
    });
    crawl().await?;

    Ok(())
}

async fn crawl() -> Res<()> {
    // rather than assume that maximum concurrency will "work out" (hint, it
    // doesn't), this program instead spawns off workers in batches, doubling
    // the count each time until we start seeing regular errors (at which point
    // we revert to prior batch count).

    let global_seen: Arc<StdMutex<Vec<Url>>> = Arc::new(StdMutex::new(Vec::new()));
    let mut site_todo: Vec<(Url, u32)> = SITES.iter()
        .filter_map(|x| Url::parse(x).ok().map(|x|(x, 0)))
        .collect();
    let mut site_len: Vec<(Url, usize)> = Vec::new();
    let mut accum_errs: Vec<(Url, OpaqueError)> = Vec::new();

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    enum BatchState {
        Incrementing,
        Doubling,
        Tuning { min: usize, max: usize },
        Stable(usize),
    }
    struct Batch { state: BatchState, size: usize }
    let mut worker_batch: Batch = Batch { state: BatchState::Incrementing, size: 1 };
    'more_todo: while site_todo.len() > 0 {
        let mut handles = Vec::new();

        // this for-loop *should* be nearly instanteous.
        // (TODO double-check. either via insrumentation or assertions)
        for _ in 0..worker_batch.size {
            if let Some((site, depth)) = site_todo.pop() {
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(3))
                    .build()?;
                let global_seen = global_seen.clone();
                handles.push(tokio::spawn(async move {
                    let mut seen = Vec::new();
                    let mut site_todo: Vec<(Url, u32)> = Vec::new();
                    let mut site_len: Vec<(Url, usize)> = Vec::new();
                    let mut accum_errs: Vec<(_, OpaqueError)> = Vec::new();
                    macro_rules! ret {
                        () => { return (site.clone(), seen, site_todo, site_len, accum_errs) }
                    }
                    macro_rules! accum_err {
                        ($input:expr) => {
                            match $input {
                                Err(err) => { ERROR_COUNT.fetch_add(1, Ordering::SeqCst); accum_errs.push((site.clone(), Box::new(err))); ret!() }
                                Ok(val) => val,
                            }
                        }
                    }
                    ATTEMPT_COUNT.fetch_add(1, Ordering::SeqCst);
                    let response = accum_err!(client.get(site.clone()).send().await);
                    RESPONSE_COUNT.fetch_add(1, Ordering::SeqCst);
                    let html = accum_err!(response.text().await);
                    DECODED_COUNT.fetch_add(1, Ordering::SeqCst);
                    let len = html.len();
                    site_len.push((site.clone(), len));
                    if depth == MAX_DEPTH {
                        BAILOUT_COUNT.fetch_add(1, Ordering::SeqCst);
                        ret!()
                    }
                    seen.push(site.clone());
                    let dom = accum_err!(Dom::parse(&html));
                    let mut node_todo = dom.children;
                    let mut node_count = 0;
                    while let Some(node) = node_todo.pop() {
                        node_count += 1;
                        match node {
                            Node::Text(_) |
                            Node::Comment(_) => {}
                            Node::Element(elem) => {
                                if let ("a", Some(&Some(href))) = (elem.name.as_str(), elem.attributes.get("href").as_ref()) {
                                    if let Ok(url) = Url::parse(href) {
                                        let global_seen = global_seen.lock().unwrap();
                                        if !global_seen.contains(&url) &&
                                            !seen.contains(&url) {
                                                // println!("pushing {}", url);
                                                site_todo.push((url.clone(), depth+1));
                                            }
                                    }
                                }
                                node_todo.extend(elem.children.into_iter());
                            }
                        }
                    }
                    // println!("{}: len {}, nodes: {}", site, len, node_count);
                    PROCESSED_COUNT.fetch_add(1, Ordering::SeqCst);
                    ret!();
                }));
            }
        }

        assert!(handles.len() <= worker_batch.size);

        HANDLE_COUNT.store(handles.len(), Ordering::SeqCst);

        // at this point, we have spawned some number of tasks. we do not know
        // if this batch size is healthy or not. (and we cannot know until we
        // give all the threads a chance to make enough progress to observe an
        // error in the first place.)
        //
        // So, we await all the threads, and then inspect how many of them ended
        // up in an error. If over 25% ends up in error, then we do not grow.

        let mut error_count = 0;
        let handles_len = handles.len();
        let mut delta_seen = 0;
        let mut delta_todo = 0;
        let mut delta_len = 0;
        let mut delta_errs = 0;

        let pre_joins = Instant::now();
        while let Some(h) = handles.pop() {
            HANDLE_COUNT.store(handles.len(), Ordering::SeqCst);
            let (site, seen, todo, len, errs) = h.await?;

            if errs.len() > 0 {
                error_count += 1;
            }
            delta_seen += seen.len();
            delta_todo += todo.len();
            delta_len += len.len();
            delta_errs += errs.len();

            {
                let mut global_seen = global_seen.lock().unwrap();
                global_seen.extend(seen.into_iter());
            }
            site_todo.extend(todo.into_iter());
            site_len.extend(len.into_iter());
            accum_errs.extend(errs.into_iter());
        }

        let post_joins = Instant::now();

        println!("handles: {} seen: {} (+{}) todo: {} (+{}) len: {} (+{}) errs: {} (+{}) time: {:?} (per-handle: {:?})",
                 handles_len,
                 global_seen.lock().unwrap().len(), delta_seen,
                 site_todo.len(), delta_todo,
                 site_len.len(), delta_len,
                 accum_errs.len(), delta_errs,
                 post_joins - pre_joins,
                 (post_joins - pre_joins)/(handles_len as u32),
        );

        let new_size;
        if worker_batch.size > 8 && error_count * 4 > handles_len {
            new_size = worker_batch.size >> 1;
            debug!("shrink (error_count: {} handles: {}) {} => {}",
                   error_count, handles_len, worker_batch.size, new_size);
        } else if worker_batch.state == BatchState::Doubling {
            new_size = worker_batch.size * 2;
            debug!("grow (error_count: {} handles: {}) {} to {}",
                   error_count, handles_len, worker_batch.size, new_size);
        } else if worker_batch.state == BatchState::Incrementing {
            new_size =  worker_batch.size + 1;
            debug!("grow (error_count: {} handles: {}) {} to {}",
                   error_count, handles_len, worker_batch.size, new_size);
            worker_batch.size += 1;
        } else {
            new_size = worker_batch.size;
        }
        worker_batch.size = new_size;
    }

    Ok(())
}

async fn crawl_absurdly_concurrent() -> Res<()> {
    let global_seen: Arc<StdMutex<Vec<Url>>> = Arc::new(StdMutex::new(Vec::new()));
    let mut global_site_todo: Vec<(Url, u32)> = SITES.iter()
        .filter_map(|x| Url::parse(x).ok().map(|x|(x, 0)))
        .collect();
    let mut global_site_len: Vec<(Url, usize)> = Vec::new();
    let mut global_accum_errs: Vec<(Url, OpaqueError)> = Vec::new();
    let concurrent_actors = Arc::new(AtomicUsize::new(0));
    #[derive(Debug)]
    struct TrackActor(Arc<AtomicUsize>);
    impl TrackActor {
        fn new(count: Arc<AtomicUsize>) -> Self {
            count.fetch_add(1, Ordering::SeqCst);
            Self(count)
        }
    }
    impl Drop for TrackActor {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }


    while global_site_todo.len() > 0 {
        let mut handles = Vec::new();
        while let Some((site, depth)) = global_site_todo.pop() {
            let global_seen = Arc::clone(&global_seen);
            let concurrent_actors = concurrent_actors.clone();
            handles.push(tokio::spawn(async move {
                let mut seen = Vec::new();
                let mut site_todo: Vec<(Url, u32)> = Vec::new();
                let mut site_len: Vec<(Url, usize)> = Vec::new();
                let mut accum_errs: Vec<(_, OpaqueError)> = Vec::new();
                macro_rules! ret {
                    () => { return (seen, site_todo, site_len, accum_errs) }
                }
                macro_rules! accum_err {
                    ($input:expr) => {
                        match $input {
                            Err(err) => { ERROR_COUNT.fetch_add(1, Ordering::SeqCst); accum_errs.push((site.clone(), Box::new(err))); ret!() }
                            Ok(val) => val,
                        }
                    }
                }
                // println!("depth {:2}; todo: {:3} fetched: {:3} seen: {:3} errs: {:3}; processing {}",
                //          depth, site_todo.len(), site_len.len(), seen.len(), accum_errs.len(), site);
                ATTEMPT_COUNT.fetch_add(1, Ordering::SeqCst);
                let response;
                let mut try_get_count = 0;
                let mut track = TrackActor::new(concurrent_actors.clone());
                loop {
                    try_get_count += 1;
                    match reqwest::get(site.clone()).await {
                        Ok(r) => {
                            response = r;
                            // println!("track: {:?}", track);
                            break;
                        }
                        Err(err) => {
                            // I have found that I am getting "too many files open" errors.
                            RETRY_COUNT.fetch_add(1, Ordering::SeqCst);
                            let too_much = concurrent_actors.load(Ordering::SeqCst);
                            if try_get_count <= 8 && too_much >= 2 {
                                drop(track);
                                let mut peek_count = try_get_count;
                                loop {
                                    peek_count += 1;
                                    let actor_count = concurrent_actors.load(Ordering::SeqCst);
                                    let sleep_time = Duration::from_millis((1 << peek_count) * rand::random::<u8>() as u64);
                                    if false {
                                        println!("sleeping before retry for {:?}; actor_count: {} too_much: {} loop_count: {:?}",
                                                 sleep_time, actor_count, too_much, (try_get_count, peek_count));
                                    }
                                    tokio::time::sleep(sleep_time).await;
                                    if actor_count * 2 < too_much {
                                        track = TrackActor::new(concurrent_actors.clone());
                                        break;
                                    }
                                }
                            } else {
                                accum_err!(Err(err))
                            }
                        }
                    }
                }
                RESPONSE_COUNT.fetch_add(1, Ordering::SeqCst);
                let html = accum_err!(response.text().await);
                DECODED_COUNT.fetch_add(1, Ordering::SeqCst);
                let len = html.len();
                site_len.push((site.clone(), len));
                if depth == MAX_DEPTH { ret!() }
                seen.push(site.clone());
                let dom = accum_err!(Dom::parse(&html));
                let mut node_todo = dom.children;
                let mut node_count = 0;
                while let Some(node) = node_todo.pop() {
                    node_count += 1;
                    match node {
                        Node::Text(_) |
                        Node::Comment(_) => {}
                        Node::Element(elem) => {
                            if let ("a", Some(&Some(href))) = (elem.name.as_str(), elem.attributes.get("href").as_ref()) {
                                if let Ok(url) = Url::parse(href) {
                                    let global_seen = global_seen.lock().unwrap();
                                    if !global_seen.contains(&url) &&
                                        !seen.contains(&url) {
                                            // println!("pushing {}", url);
                                            site_todo.push((url.clone(), depth+1));
                                        }
                                }
                            }
                            node_todo.extend(elem.children.into_iter());
                        }
                    }
                }
                // println!("{}: len {}, nodes: {}", site, len, node_count);
                PROCESSED_COUNT.fetch_add(1, Ordering::SeqCst);
                (seen, site_todo, site_len, accum_errs)
            }));
        }

        HANDLE_COUNT.store(handles.len(), Ordering::SeqCst);

        for (j, handle) in handles.into_iter().enumerate() {
            let (seen, site_todo, site_len, accum_errs) = match handle.await {
                Ok(vecs) => vecs,
                Err(err) => return Err(Box::new(err)),
            };

            if accum_errs.len() > 0 {
                println!("midway through pass; handle {} accum_errs: {} {:?}",
                         j, accum_errs.len(),
                         accum_errs.iter().take(3).collect::<Vec<_>>())
            }

            {
                let mut global_seen = global_seen.lock().unwrap();
                global_seen.extend(seen.into_iter());
            }
            global_site_todo.extend(site_todo.into_iter());
            global_site_len.extend(site_len.into_iter());
            global_accum_errs.extend(accum_errs.into_iter());
        }
        println!("finished pass; site_todo: {} site_len: {} accum_errs: {}",
                 global_site_todo.len(),
                 global_site_len.len(),
                 global_accum_errs.len());
    }

    Ok(())
}

/*
async fn really_sync_main_singly_threaded() -> Res<()> {

    let handle = std::thread::spawn(|| -> Res<()> {
        let mut seen: Vec<Url> = Vec::new();
        let mut site_todo: Vec<(Url, u32)> = SITES.iter()
            .filter_map(|x| Url::parse(x).ok().map(|x|(x, 0)))
            .collect();
        let mut site_len: Vec<(Url, usize)> = Vec::new();
        let mut accum_errs: Vec<(Url, OpaqueError)> = Vec::new();
        const MAX_DEPTH: u32 = 2;
        while let Some((site, depth)) = site_todo.pop() {
            macro_rules! accum_err {
                ($input:expr) => {
                    match $input {
                        Err(err) => { accum_errs.push((site.clone(), Box::new(err))); continue }
                        Ok(val) => val,
                    }
                }
            }
            println!("depth {:2}; todo: {:3} fetched: {:3} seen: {:3} errs: {:3}; processing {}",
                     depth, site_todo.len(), site_len.len(), seen.len(), accum_errs.len(), site);
            let response = accum_err!(reqwest::blocking::get(site.clone()));
            let html = accum_err!(response.text());
            let len = html.len();
            site_len.push((site.clone(), len));
            seen.push(site.clone());
            let dom = accum_err!(Dom::parse(&html));
            let mut node_todo = dom.children;
            let mut node_count = 0;
            while let Some(node) = node_todo.pop() {
                node_count += 1;
                match node {
                    Node::Text(_) |
                    Node::Comment(_) => {}
                    Node::Element(elem) => {
                        if let ("a", Some(&Some(href))) = (elem.name.as_str(), elem.attributes.get("href").as_ref()) {
                            if let Ok(url) = Url::parse(href) {
                                if depth < MAX_DEPTH {
                                    if !seen.contains(&url) {
                                        println!("pushing {}", url);
                                        site_todo.push((url.clone(), depth+1));
                                    }
                                }
                            }
                        }
                        node_todo.extend(elem.children.into_iter());
                    }
                }
            }
            println!("{}: len {}, nodes: {}", site, len, node_count);
        }

        Ok(())
    });

    let () = match handle.join() {
        Ok(res) => res?,
        Err(join_err) => std::panic::resume_unwind(join_err),
    };

    Ok(())
}
 */

#[tokio::test]
async fn my_test() {
    println!("Hello, world from my_test");
}
