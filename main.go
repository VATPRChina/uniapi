package main

import (
	"fmt"
	"io/fs"
	"log"

	"github.com/danielgtaylor/huma/v2"
	"github.com/danielgtaylor/huma/v2/adapters/humafiber"
	"github.com/gofiber/fiber/v2"
	"github.com/gookit/config/v2"
	"github.com/gookit/config/v2/toml"
	"github.com/vatprchina/uniapi/controller"
	"github.com/vatprchina/uniapi/external"
)

func main() {
	config.AddDriver(toml.Driver)
	err := config.LoadFiles("config.toml")
	fsErr, isFsPathError := err.(*fs.PathError)
	fmt.Println(fsErr)
	if err != nil && !isFsPathError {
		panic(err)
	}

	external.DatabaseConnect()

	app := fiber.New()

	api := humafiber.New(app, huma.DefaultConfig("VATPRC UniAPI", "v1"))

	controller.AddGreet(api)

	log.Fatal(app.Listen(":3000"))
}
