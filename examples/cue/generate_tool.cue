package config

import (
	"encoding/yaml"
	"tool/cli"
	"tool/file"
)

_path: "specs"

command: clean: {
	clean: file.RemoveAll & {
		path: _path
	}
}

command: generate: {
	clean: file.RemoveAll & {
		path: _path
	}

	mkdir: file.MkdirAll & {
		$dep: clean.$done
		path: _path
	}

	for name, resources in _resources {
		(name): {
			write: file.Create & {
				$dep:     mkdir.$done
				filename: "\(_path)/\(name).yaml"
				contents: yaml.MarshalStream(resources)
			}

			print: cli.Print & {
				$dep: write.$done
				text: "\(name) created"
			}
		}
	}
}
