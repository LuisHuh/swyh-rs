# swyh-rs Changelog

- 1.3.22 (unreleased)

- 1.3.21 (Dec 8 2021 dheijl)
  - get rid of all remaining traces of Range Headers (Linn) code  
  - fix panic when reading config after upgrade from 1.3.12 or earlier (thanks @FinalSh4re)

- 1.3.20 (Nov 24 2021 dheijl)
  - (experimental) 24 bit LPCM (audio/L24) support
  - get rid of the ini file format in favour of toml, so that I can use serde (de)serialization instead of reading and writing individual values
  - automatically migrate an exisiting config.ini to config.toml
  - update to Rust edition 2021
  - wait for the first SSDP discovery to complete before starting the streaming server
  - disable the terminal logger on Windows release build, as it panics with Rust 2021
  - add an "Accept-Ranges : none" header to HTTP responses as HTTP ranges (Linn!) are not supported
  - update dependencies

- 1.3.19 (July 6 2021 dheijl)
  - rearrange UI
  - bugfix: forgot to save the new last_network config value on first start

- 1.3.18 (July 2 2021 dheijl)
  - fix button insert position

- 1.3.17 (July 2 2021 dheijl)
  - log streaming request headers in debug log ([issue #40](https://github.com/dheijl/swyh-rs/issues/40))
  - add buildall script and 32-bit Windows build
  - add option to select the network interface (IPV4) to use and save it in the config

- 1.3.16 (May 16 2021 dheijl)
  - remove simultaneous streaming limit and reduce thread count
  - fix renderer button header and button index position

- 1.3.14 (Apr 28 2021 dheijl)
  - upgrade to fltk-rs 1.x
  - include Ubuntu (Mint 20.1) binary in release

- 1.3.13 (Apr 13 2021 dheijl)
  - update SimpleLog
  - add configurable HTTP listener port number

- 1.3.12 (Mar 23 2021 dheijl)
  - latest icon versions by @numanair

- 1.3.11 (Mar 21 20121 dheijl)
  - note-only icon for smaller icon sizes designed by @numanair

- 1.3.10 (Mar 16 2021 dheijl)
  - added icon designed by @numanair

- 1.3.9 (Mar 14 2021 dheijl)
  - clear rms meter widget values when checkbox is (un)set
  - restructure more code into modules (ui, audio), and some refactoring

- 1.3.8 (Feb 27 2021 dheijl)
  - show left and right channel RMS values

- 1.3.7 (Feb 25 2021 dheijl)
  - use ParkingLot RwLock instead of Mutex since most accesses of the locks (CLIENTS, CONFIG) are read anyway
  - clean-up configuration code
  - upgrade to rustc 1.50

- 1.3.6 (Feb 21 2021 dheijl)
  - migrate the configuration folder from `$HOME/swyh-rs` to `$HOME/.swyh-rs` so that it is hidden on Linux and comes before normal folders in Windows Explorer ([issue #32](https://github.com/dheijl/swyh-rs/issues/32))
  - add visual feedback (RMS value) for the audio capture
  - add InnoSetup Windows Setup, unsigned
  
- 1.3.5 (Feb 18 2021 dheijl)
  - changes for the new app::awake() in fltk-rs 0.14.0
  - deglob imports
  - optional support for WAV (audio/wma) file format of infinite length for renderers that do not support "naked" PCM

- 1.3.4 (Feb 03 2021 dheijl)
  - optimize GUI event loop with new fltk-rs app messages, decreasing CPU usage even more

- 1.3.3 (Jan 31 2021 dheijl)
  - remove redundant closures
  - better resizing with fltk-rs thanks @Moalyousef
  - use tiny-http crate instead of github repo (identity-encoding fix included)

- 1.3.2 (Jan 7 2020 dheijl)
  - prevent panics caused by changed ureq 2.0 error handling
  - implement a global configuration singleton (read once at startup) so that we don't have to reread it every time
  - cargo clippy
  - allow for multiple streaming connections to exist for the same renderer. This should finally fix the problems with Autoresume getting into a play/stop play loop with some renderers.

- 1.3.1 (Jan 6 2020 dheijl)
  - upgrade to rust 1.49
  - fix capture timeouts for Bubble with OpenHome Chromecast/Nest Audio

- 1.3.0 (Jan 4 2021 dheijl)
  - Removed the "SeekId" action from OpenHome control, as it is not needed and interferes with AutoResume on some renderers (Bubble)
  - adjusted the capture time-out to be smaller (15 sec) than the "no sound" time-out (30 sec) of BubbleUPNP Server
  - with the above changes Autoresume should now work reliably with OpenHome and Bubble UPNP Server
  - upgrade ureq to 2.0 (comes with breaking changes)

- 1.2.2 (Dec 29 2020 dheijl)
  - send continuous silence if no sound has been captured for 30 seconds to prevent connected renderers disconnecting
  - use official github tiny-http repo now that Equality_Reader is removed from Identity transfer

- 1.2.1 (Dec 17 2020 dheijl)
  - fix copy-and-paste bug when reading configuration file

- 1.2.0 (Dec 14 2020 dheijl)
  - slight GUI changes (BG color)
  - replace a couple of fltk handle2() events by callback2() events
  - some code cleanup

- 1.1.1 (Dec 7 2020 dheijl)
  - fix renderer button insert position

- 1.1.0 (Dec 7 2020 dheijl)
  - use good practice for Cargo.toml and Cargo.lock files (thanks @Boscop)
  - option to disable chunked transfer encoding in cases where the (AVTransport) renderer has problems with it
  (*__Note__: I had to use a patched fork of tiny_http to let this work, as per [this pull request](https://github.com/tiny-http/tiny-http/pull/183)*)

- 1.0.8 (Nov 27 2020 dheijl)
  - switch to parking_lot Mutex and Once, and use Ninja-Build for fltk to speed up CMake in the fltk build
  
- 1.0.7 (Nov 19 2020 dheijl)
  - upgrade to rustc 1.48, fltk-rs 0.10.11, and some small code improvements

- 1.0.6 (Nov 17 2020 dheijl)
  - implement autoconnect to the last used renderer on startup (<https://github.com/dheijl/swyh-rs/issues/19>)

- 1.0.5 (Nov 17 2020 dheijl)
  - various code improvements offered by @Boscop (<https://github.com/dheijl/swyh-rs/issues/22>)

- 1.0.4 (Nov 16 2020 dheijl)
  - bugfix for sample rate from default audio output device being advertised while sample rate of actual audio output device was used

- 1.0.3 (Nov 16 2020 dheijl)
  - SSDP now detects all OpenHome and DLNA renderers, but only uses the OpenHome device for devices that are capable of both
    - prevent panic in audio source chooser caused by vertical bar ("|") in audio source name, it too must be escaped for FLTK Menu_Item...

- 1.0.2 (Nov 15 2020 dheijl)
  - support for Chromecast as DLNA device defined in Bubble UPNP Server, thanks BubbleSoft for the assistance!

- 1.0.1 (Nov 14 2020 dheijl & MoAlyousef)
  - resizing is now usable (except for the horizontal scrollbar at the bottom that may get lost)
  - fix for '/' in the name of an output audio source

- 1.0.0 (Nov 11 2020 dheijl)
    enable windows resizing again, but it does not really work in FLTK, even when using Pack groups...

- 0.9.9  (Nov 11 2020 dheijl)
    disable resizing

- 0.9.8 (Nov 10 2020 dheijl)
    better handling of ssdp discovery change and restart button

- 0.9.7  (Nov 9 2020  dheijl)
    show a restart button after a configuration change that needs an application restart

- 0.9.6  (Nov 9 2020  dheijl)
    improve application start time

- 0.9.5  (Nov 8 2020  dheijl)
    make the SSDP discovery interval a configurable option

- 0.9.4  (Nov 6 2020  dheijl)
    simplify and unify SSDP discovery

- 0.9.3  (Oct 21 2020 dheijl)
    reduce network traffic during SSDP discovery for previously discovered renderers

- 0.9.2  (Oct 20 2020 dheijl)
    rerun SSDP discovery every minute, updating the renderers
