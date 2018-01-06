package config

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	homedir "github.com/mitchellh/go-homedir"
)

func GetDefaultConfigDir() (dir string, err error) {
	home, err := homedir.Dir()
	if err != nil {
		return "", err
	}
	dir = filepath.Join(home, ".config", "cpm")
	return dir, nil
}

func GetDefaultRootDir() (dir string, err error) {
	home, err := homedir.Dir()
	if err != nil {
		return "", err
	}
	dir = filepath.Join(home, ".cpm")
	return dir, nil
}

func CreateDefaultConfigDir() (dir string, err error) {
	dir, err = GetDefaultConfigDir()
	if err := os.MkdirAll(dir, 0700); err != nil {
		return "", fmt.Errorf("Can not create directory: %v", err)
	}
	return dir, nil
}

func CreateDefaultConfigFile() (file string, err error) {

	dir, err := GetDefaultConfigDir()
	if err != nil {
		return "", err
	}
	file = filepath.Join(dir, "config.json")
	//file already exists
	if _, err := os.Stat(file); err == nil {
		return "", nil
	}

	f, err := os.OpenFile(file, os.O_WRONLY|os.O_CREATE, 0666)
	if err != nil {
		return "", err
	}
	encoder := json.NewEncoder(f)
	encoder.SetIndent("", "   ")

	root, err := GetDefaultRootDir()
	if err != nil {
		return "", nil
	}

	config := map[string]string{
		"root": root,
	}
	err = encoder.Encode(config)
	if err != nil {
		return "", nil
	}
	return file, nil
}
