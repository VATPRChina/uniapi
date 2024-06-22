package main

import (
	"log"

	"github.com/danielgtaylor/huma/v2"
	"github.com/danielgtaylor/huma/v2/adapters/humafiber"
	"github.com/gofiber/fiber/v2"
	"github.com/vatprchina/uniapi/controller"
	"github.com/vatprchina/uniapi/external"
	"github.com/vatprchina/uniapi/util"
)

func main() {
	util.LoadConfig()

	external.DatabaseConnect()

	app := fiber.New()

	api := humafiber.New(app, huma.DefaultConfig("VATPRC UniAPI", "v1"))

	controller.AddRoutes(api)

	log.Fatal(app.Listen(":3000"))
}
