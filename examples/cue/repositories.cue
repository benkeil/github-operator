package config

import "strings"

_resources: [string]: [...#CR]
_resources: {for repo in #repositories {
	(strings.Split(repo, "_")[1]): [
		#Repository & {
			metadata: {
				name:      strings.Split(repo, "_")[1]
				namespace: #namespace
			}
			spec: {
				full_name:              repo
				delete_branch_on_merge: true
				security_and_analysis: {
					advanced_security: {
						status: "enabled"
					}
					security_and_analysis: {
						status: "enabled"
					}
					secret_scanning_push_protection: {
						status: "enabled"
					}
					dependabot_security_updates: {
						status: "enabled"
					}
					secret_scanning_validity_checks: {
						status: "enabled"
					}
				}
			}
		},
		#AutolinkReference & {
			metadata: {
				name:      strings.Split(repo, "_")[1]
				namespace: #namespace
			}
			spec: {
				key_prefix:      "DV-"
				url_template:    "https://otto-eg.atlassian.net/browse/DV-<num>"
				is_alphanumeric: false
			}
		},
	]
}}
