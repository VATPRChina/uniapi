#!/usr/bin/env just --justfile

default: watch

build:
  dotnet build
run:
  dotnet run --project Net.Vatprc.Uniapi
watch:
  dotnet watch run --project Net.Vatprc.Uniapi
test:
  dotnet test /p:CollectCoverage=true
db-update +ARGS="":
  dotnet ef database update --project Net.Vatprc.Uniapi {{ARGS}}
new-migration NAME +ARGS="":
  dotnet ef migrations add --project Net.Vatprc.Uniapi {{NAME}} {{ARGS}}
add NAME:
  dotnet add Net.Vatprc.Uniapi package {{NAME}}
