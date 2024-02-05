package config

import (
	"strings"
	"encoding/yaml"
)

#repositories: [
	"otto-ec/pdh-da_alarm-notification",
	"otto-ec/pdh-da_customer-popularity",
]
#namespace: "pdh-da"

_repositories: [...#GitHubRepository]
_repositories: [for repo in #repositories {
	{
		metadata: {
			name:      strings.Split(repo, "_")[1]
			namespace: #namespace
		}
		spec: {
			full_name: repo
			repository: {
				delete_branch_on_merge: true
				security_and_analysis: {
					advanced_security: {
						stauts: "enabled"
					}
					security_and_analysis: {
						stauts: "enabled"
					}
					secret_scanning_push_protection: {
						stauts: "enabled"
					}
					dependabot_security_updates: {
						stauts: "enabled"
					}
					secret_scanning_validity_checks: {
						stauts: "enabled"
					}
				}
			}
			autolink_references: [
				{
					key_prefix:      "DV-"
					url_template:    "https://otto-eg.atlassian.net/browse/DV-<num>"
					is_alphanumeric: false
				},
			]
		}
	}
}]

specs: yaml.MarshalStream(_repositories)
