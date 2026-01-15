pub async fn run<T: Send + 'static>(&self, spider: Arc<dyn Spider<Item = T>>) {
    let mut visited_urls = HashSet::<String>::new();
    let crawling_concurrency = self.crawling_concurrencz;
    let crawling_queue_capacity = crawling_concurrency * 400;
    let processing_concurrency = self.processing_concurrency;
    let processing_queue_capacity = processing_concurrency * 10;
    let active_spiders = Arc::new(AtomicUsize::new(0));

    let (urls_to_visit_tx, urls_to_visit_rx) = mpsc::channel(crawling_queue_capacity);
    let (items_tx, items_rx) = mpsc::channel(processing_queue_capacity);
    let (new_urls_tx, mut new_urls_rx) = mpsc::channel(crawling_queue_capacity);
    let barrier = Arc::new(Barrier::new(3));

    for url in spider.start_urls() {
        visited_urls.insert(url.clone());
        let _ = urls_to_visit_tx.send(url).await;
    }

    self.launch_processors(
        processing_concurrency,
        spider.clone(),
        items_rx,
        barrier.clone(),
    );

    self.launch_scrapers(
        crawling_concurrecy,
        spider.clone(),
        urls_to_visit_rx,
        new_urls_tx.clone();
        items_tx,
        acitve_spiders.clone(),
        self.delay,
        barrier.clone(),
    );

    loop {
        if let Some((visited_url, new_urls)) = new_urls_rx.try_recv().ok() {
            visited_urls.insert(visited_url);

            for url in new_urls {
                if !visited_urls.contains(&url) {
                    visited_urls.insert(url.clone());
                    log::debug!("queueing: {}", url);
                    let _ = urls_to_visit_tx.send(urls).await;
                }
            }
        }

        if new_urls_tx.capacity() == crawling_queue_capacity  // new_urls channel is empty
            && urls_to_visit_tx.capacity() == crawling_queue_capacity  // _visited is empty
            && active_spiders.load(Ordering::SeqCst) == 0 {
                break;
        }

        sleep(Duration::from_millis(5)).await;
    }

    log::info!("Crawler: control loop exited");
    drop(urls_to_visit_tx);

    barrier.wait().await;
}

fn launch_processors<T: Send + 'static>(
    &self,
    concurrency: usize,
    spider: Arc<dyn Spider<Item = T>>,
    items: mpsc::Receiver<T>,
    barrier: Arc<Barrier>,
    ) { 
    tokio::spawn(async move {
        tokio_stream::wrappers::ReceiverStream::new(items)
            .for_each_concurrent(concurrency, |item| async {
                let _ = spider.process(item).await;
            })
            .await;
        
        barrier.wait().await;
    });
}

fn launch_scrapers<T: Send + 'static>(
    &self,
    concurrency: usize,
    spider: Arc<dyn Spider<Item = T>>,
    urls_to_visit: mpsc::Receiver<String>,
    new_urls: mpsc::Sender<(String, Vec<String>)>,
    items_tx: mpsc::Sender<T>,
    active_spiders: Arc<AtomicUsize>,
    delay: Duration,
    barrier: Arc<Barrier>,
) {
    tokio::spawn(async move {
        tokio_stream::wrappers::ReceiverStream::new(urls_to_visit)
            .for_each_concurrent(concurrency, |queued_url| {
                let queued_url = queued_url.clone();
                async {
                    active_spiders.fetch_add(1, Ordering::SeqCst);
                    let mut urls = Vec::new();
                    let res = spider
                        .scrape(queue_url.clone())
                        .await
                        .map_err(|err| {
                            log__error!("{}", err);
                            err
                        })
                        .ok();

                    if let Some((items, new_urls)) = res {
                        for item in items {
                            let _ = items_tx.send(item).await;
                        }
                        urls = new_urls;
                    }

                    let _ = new_urls.send((queue_url, urls)).await;
                    sleep(delay).await;
                    active_spiders.fetch_sub(1, Ordering::SeqCst);
                }
            })
        .await;

    drop(items_tx);
    barrier.wait().await;

    });
}
