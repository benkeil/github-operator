package config

import (
	"encoding/yaml"
	"tool/cli"
	"tool/file"
)

#path: "specs"

command: clean: {
	clean: file.RemoveAll & {
		path: #path
	}
}

command: generate: {
	for name, resources in _resources {
		(name): {
			clean: file.RemoveAll & {
				path: #path
			}

			mkdir: file.MkdirAll & {
				$dep: clean.$done
				path: #path
			}

			write: file.Create & {
				$dep:     mkdir.$done
				filename: "\(#path)/\(name).yaml"
				contents: yaml.MarshalStream(resources)
			}

			print: cli.Print & {
				$dep: write.$done
				text: "\(name) created"
			}
		}
	}
}
