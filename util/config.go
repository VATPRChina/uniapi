package util

import (
	"log"

	"github.com/go-playground/validator/v10"
	"github.com/spf13/viper"
)

type config struct {
	Discourse struct {
		ApiKey      string `mapstructure:"api_key" validate:"required"`
		ApiUsername string `mapstructure:"api_username" validate:"required"`
	}
	Database struct {
		Uri string `mapstructure:"uri" validate:"required"`
	}
}

var Config config

func LoadConfig() {
	viper.SetConfigName("config")
	viper.SetConfigType("toml")
	viper.AddConfigPath(".")
	viper.SetEnvPrefix("VATPRC")
	viper.AutomaticEnv()

	viper.SetDefault("database.uri", "host=localhost dbname=uniapi")

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			// ignore file not found
		} else {
			log.Fatalf("Error reading config file, %v", err)
		}
	}

	if err := viper.Unmarshal(&Config); err != nil {
		log.Fatalf("unable to decode into struct, %v", err)
	}

	validate := validator.New(validator.WithRequiredStructEnabled())
	if err := validate.Struct(Config); err != nil {
		log.Fatalf("invalid config, %v, %v", err, Config)
	}
}
