package config

#CR: #Repository | #AutolinkReference | #RepositoryPermission

#Repository: {
	apiVersion: "github.platform.benkeil.de/v1alpha1"
	kind:       "Repository"
	metadata:   #Metadata
	spec:       #RepositorySpec
}

#Metadata: {
	name:      string
	namespace: string
}

#RepositorySpec: {
	full_name:               string
	delete_branch_on_merge?: bool
	security_and_analysis?: {
		advanced_security?:               #SecurityAndAnalysisStatus
		security_and_analysis?:           #SecurityAndAnalysisStatus
		secret_scanning_push_protection?: #SecurityAndAnalysisStatus
		dependabot_security_updates?:     #SecurityAndAnalysisStatus
		secret_scanning_validity_checks?: #SecurityAndAnalysisStatus
	}
}

#SecurityAndAnalysisStatus: {
	status: "enabled" | "disabled"
}

#AutolinkReference: {
	apiVersion: "github.platform.benkeil.de/v1alpha1"
	kind:       "AutolinkReference"
	metadata:   #Metadata
	spec:       #AutolinkReferenceEntry
}

#AutolinkReferenceEntry: {
	key_prefix:      string
	url_template:    string
	is_alphanumeric: bool
}

#RepositoryPermission: {
	apiVersion: "github.platform.benkeil.de/v1alpha1"
	kind:       "RepositoryPermission"
	metadata:   #Metadata
	spec:       #RepositoryPermissionSpec
}

#RepositoryPermissionSpec: {
	full_name:      string
	full_team_name: string
	permission:     string
}
