To compile your wxWidgets c++ code and link with wxWidgets in build.rs files.

``` 
// Will compile all cpp files in my_cpp_folder.
// 
// MyApp is name of my class extending wxApp
// 
wxbuild_rs:build("my_cpp_folder",true, "MyApp");
``` 

Environment variables:

WX_CONFIG - path to wx-config script, if not set it will require wx-config to be present in $PATH

WX_DIR - on windows, path to wxWidgets folder. It assumes it was compiled static for x64 with VC.
