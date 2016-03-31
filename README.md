# TODO

- try wrapping Directory in a RwLock or just the varianble contents?
- make Directory use crossbeam to avoid all the mutexes
- try using a crossbeam::sync::MsQueue instead of channels to send data when new item found in directory
- try again to get the curses stuff into a thread??
- get multithreaded scanning to work properly again
- know how many pending filter events are in play and when 0 allow blocking of main thread when
  waiting for keyboard input
- can the continuous filter return a reference to the FilteredDirectory??
