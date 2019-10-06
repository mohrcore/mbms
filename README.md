# mbms
## Rust crate for reading and manipulating Be-Music Source files.
Thi is work in progress. The capabilities are very limited and API will probably go trough some major changes during development

You cna fid example usage in main.rs

### What the crate can do for now
- Open .bms files and parse channel commands
- Read BPM from from charts with single BPM (it is possible to manually add BPM changes tho)
- Read SOME metadat from charts
- Convert chart time (in measures) to absolute time (in seconds) and vice-versa
- Iterate through charts content
- Print measures from charts for debugging purpouses
- Load WAV resource paths from BMS

### TODO List:
- Write docs
- Add support for changing bpm
- Add support for long/charge notes
- Add support for P2 charts
- Implement chartlogic evaluation
- Make WBMS structure for easy chart editing
- Implemend saving as BMS
- Many more

### Contributing
There are no contribution rules as for now as the crate is in very early development stage.
Feel free to propose any changes tho.