package config

#GitHubRepository: {
	apiVersion: "platform.benkeil.de/v1alpha1"
	kind:       "GitHubRepository"
	metadata:   #Metadata
	spec:       #GitHubRepositorySpec
}

#Metadata: {
	name:      string
	namespace: string
}

#GitHubRepositorySpec: {
	full_name: string
	repository?: {
		delete_branch_on_merge?: bool
		security_and_analysis?: {
			advanced_security?:               #SecurityAndAnalysisStatus
			security_and_analysis?:           #SecurityAndAnalysisStatus
			secret_scanning_push_protection?: #SecurityAndAnalysisStatus
			dependabot_security_updates?:     #SecurityAndAnalysisStatus
			secret_scanning_validity_checks?: #SecurityAndAnalysisStatus
		}
	}
	autolink_references?: [#AutolinkReference]
}

#SecurityAndAnalysisStatus: {
	stauts: "enabled" | "disabled"
}

#AutolinkReference: {
	key_prefix:      string
	url_template:    string
	is_alphanumeric: bool
}
