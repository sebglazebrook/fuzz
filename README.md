#TODO

- try wrapping Directory in a RwLock or just the varianble contents?
- try using a crossbeam::sync::MsQueue instead of channels to send data when new item found in directory
- when reading new filter strings make sure the most recent one is used and old ones discarded
- improve filtering so when the regex is additive it doesn't refilter all the things just the previous matches
- try again to get the curses stuff into a thread??
- get multithreaded scanning to work properly again
- know how many pending filter events are in play and when 0 allow blocking of main thread when
  waiting for keyboard input
