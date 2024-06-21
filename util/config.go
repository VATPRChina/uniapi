package util

import (
	"crypto/ecdsa"
	"crypto/x509"
	"encoding/pem"
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
	Jwt struct {
		PrivateKey       string `mapstructure:"private_key" validate:"required"`
		PublicKey        string `mapstructure:"public_key" validate:"required"`
		PrivateKeyParsed *ecdsa.PrivateKey
		PublicKeyParsed  *ecdsa.PublicKey
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

	blockPriv, _ := pem.Decode([]byte(Config.Jwt.PrivateKey))
	x509EncodedPriv := blockPriv.Bytes
	privateKey, err := x509.ParsePKCS8PrivateKey(x509EncodedPriv)
	if err != nil {
		log.Fatalf("failed to parse private key (Config.Jwt.PrivateKey), %v", err)
	}
	if _, ok := privateKey.(*ecdsa.PrivateKey); !ok {
		log.Fatalf("private key (Config.Jwt.PrivateKey) is not ECDSA")
	}
	Config.Jwt.PrivateKeyParsed = privateKey.(*ecdsa.PrivateKey)

	blockPub, _ := pem.Decode([]byte(Config.Jwt.PublicKey))
	x509EncodedPub := blockPub.Bytes
	publicKey, err := x509.ParsePKIXPublicKey(x509EncodedPub)
	if err != nil {
		log.Fatalf("failed to parse public key (Config.Jwt.PublicKey), %v", err)
	}
	if _, ok := publicKey.(*ecdsa.PublicKey); !ok {
		log.Fatalf("public key (Config.Jwt.PublicKey) is not ECDSA")
	}
	Config.Jwt.PublicKeyParsed = publicKey.(*ecdsa.PublicKey)
}
