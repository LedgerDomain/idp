# idp-gui

GUI browser for Indoor Data Plumbing -- this is an early and incomplete POC with terrible engineer-designed UI/UX, so please judge it on that basis only.

## Notes

On my machine, it's necessary to specify env var `WGPU_BACKEND=gl` or `idp-gui` will crash.

## To-dos

-   Look into using https://github.com/AlisCode/iced-pancurses to create a TUI version of this app.
-   Look into using https://crates.io/crates/plotters-iced for plots.
-   Look into using https://crates.io/crates/iced_focus for widget focus.
