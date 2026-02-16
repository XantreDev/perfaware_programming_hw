let dir_type = "./build" | path type
if dir_type == null or dir_type != "dir" {
    mkdir build
}

clang -g ./source/main.c -o build/main
