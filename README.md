To compile your wxWidgets c++ code and link with wxWidgets in build.rs files.

``` 
wxbuild_rs:build("my_cpp_folder",true);
``` 

Environment variables:

WX_CONFIG - path to wx-config script, if not set it will require wx-config to be present in $PATH

WX_DIR - on windows, path to wxWidgets folder. It assumes it was compiled static for x64 with VC.
