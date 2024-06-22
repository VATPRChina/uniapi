package util

import (
	"crypto/ecdsa"
	"log"
	"time"

	"github.com/go-playground/validator/v10"
	"github.com/golang-jwt/jwt/v5"
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
	Jwt struct {
		PrivateKey       string `mapstructure:"private_key" validate:"required"`
		PublicKey        string `mapstructure:"public_key" validate:"required"`
		PrivateKeyParsed *ecdsa.PrivateKey
		PublicKeyParsed  *ecdsa.PublicKey
		Issuer           string        `validate:"required"`
		Duration         time.Duration `validate:"required"`
	}
	AdminCids []string `mapstructure:"admin_cids"`
}

var Config config

func LoadConfig() {
	viper.SetConfigName("config")
	viper.SetConfigType("toml")
	viper.AddConfigPath(".")
	viper.SetEnvPrefix("VATPRC")
	viper.AutomaticEnv()

	viper.SetDefault("database.uri", "host=localhost dbname=uniapi")
	viper.SetDefault("jwt.issuer", "https://uniapi.vatprc.net")
	viper.SetDefault("jwt.duration", "1h")

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
		log.Fatalf("invalid config, %v", err)
	}

	pub, err := jwt.ParseECPublicKeyFromPEM([]byte(Config.Jwt.PublicKey))
	if err != nil {
		log.Fatalf("failed to parse public key (Config.Jwt.PublicKey), %v", err)
	}
	Config.Jwt.PublicKeyParsed = pub

	priv, err := jwt.ParseECPrivateKeyFromPEM([]byte(Config.Jwt.PrivateKey))
	if err != nil {
		log.Fatalf("failed to parse private key (Config.Jwt.PrivateKey), %v", err)
	}
	Config.Jwt.PrivateKeyParsed = priv
}
