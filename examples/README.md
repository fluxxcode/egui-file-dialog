# Examples

## Custom Filesystem

Example showing how to use the file dialog with a custom aka. virutal file system.

```shell
cargo run --example custom_filesystem
```

## Custom Right Panel

Example showing how to render custom UI inside the file dialog using the right panel.

```shell
cargo run --example custom_right_panel
```

## Multi Selection

Example showing how to select multiple files and folders at once.

```shell
cargo run --example multi_selection
```

![Screenshot](../media/examples/multi_selection.png)

## Multilingual

Example that shows how the dialog can be displayed and used in different languages.

```shell
cargo run --example multilingual
```

![Screenshot](../media/examples/multilingual.png)

## Multiple Actions

This example shows how you can query multiple files from the user in one view.

```shell
cargo run --example multiple_actions
```

![Screenshot](../media/examples/multiple_actions.png)

## Persistence

This example uses eframe to show how the persistent data of the file dialog can be saved. \
The example uses the `serde` feature to serialize the required data.

```shell
cargo run --example persistence
```

## Pick Directory

Example showing how to select a directory using the file dialog.

```shell
cargo run --example pick_directory
```

![Screenshot](../media/examples/pick_directory.png)

## Pick File

Example showing how to select a file using the file dialog.

```shell
cargo run --example pick_file
```

![Screenshot](../media/examples/pick_file.png)

## Pick File with Information View

Example showing how to pick a file and display file information using the `InformationView`.

Requires the feature `information_view` as well as these dependencies:

```toml
[dependencies]
egui-file-dialog = { version = "*", features = ["information_view"] }
egui_extras = { version = "0.30", features = ["all_loaders"] }
# required by the egui loaders
image = { version = "0.25.5", features = ["bmp", "jpeg", "gif", "png", "tiff", "rayon"] }
```

```shell
cargo run --example pick_file_with_information_view
```

![Screenshot](../media/examples/information_view.png)

## Sandbox

Sandbox app used during development of the file dialog.

```shell
cargo run --example sandbox
```

## Save File

Example showing how to save a file using the file dialog.

```shell
cargo run --example save_file
```

![Screenshot](../media/examples/save_file.png)
