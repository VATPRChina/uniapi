#!/usr/bin/env just --justfile

default: watch

build:
  dotnet build
run:
  dotnet run --project src
watch:
  dotnet watch run --project src
test:
  dotnet test /p:CollectCoverage=true
db-update +ARGS="":
  dotnet ef database update --project src {{ARGS}}
new-migration NAME +ARGS="":
  dotnet ef migrations add --project src {{NAME}} {{ARGS}}
add NAME:
  dotnet add src package {{NAME}}
