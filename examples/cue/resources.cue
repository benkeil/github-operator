package config

import "strings"

_resources: [string]: [...#CR]
_resources: {for repo in #repositories {

	let #metadata = {
		name:      strings.Split(repo, "_")[1]
		namespace: #namespace
	}

	(strings.Split(repo, "_")[1]): [
		#Repository & {
			metadata: #metadata
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
			metadata: #metadata
			spec: {
				key_prefix:      "DV-"
				url_template:    "https://otto-eg.atlassian.net/browse/DV-<num>"
				is_alphanumeric: false
			}
		},
		#RepositoryPermission & {
			metadata: #metadata
			spec: {
				full_name:      "otto-ec/pdh-da_alarm-notification"
				full_team_name: "otto-ec/pdh-distribution-analytics"
				permission:     "admin"
			}
		},
	]
}}
