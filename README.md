# TODO

- try again to get the curses stuff into a thread??

- can the continuous filter return a reference to the FilteredDirectory??
- send filter matches as events and sort/aggregate them outside of the filter
- store filtered results as dir tree to help speed up further filtering
- display a loading/processing spinner
- add colours to the output
- add benchmarking to see where there is room for speed improvements
- when refiltering, need to make sure the selected result doesn't change value

# BUGS

- Sometimes if you start filtering a large directory set straight away it might not find your file
- Sometimes when loading a large directory set for the first time it shows things like 11657/13878 and never get's
  till 100% matches until you filter and then remove the filter
- if new directory items are found and there is a filter string, matching new items are not added to the results view
