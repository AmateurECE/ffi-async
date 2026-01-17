cmake_minimum_required(VERSION 3.25)
project(app LANGUAGES C)

add_library(app
  src/app.c
  src/await.c
  src/syscalls.c
  src/sysmem.c)

target_include_directories(app PUBLIC src)

install(TARGETS app)
