# https://sagiegurari.github.io/cargo-make/#usage-task-command-script-task

kind-create:
	kind create cluster

kind-delete:
	kind delete cluster

crd-clean:
	rm -f crd/*

crd-generate: crd-clean
	mkdir -p crd/
	cargo run --bin crdgen > crd/github-repository.crd.yaml

crd-install:
	kubectl apply -f crd/

crd-delete:
	kubectl delete --ignore-not-found -f crd/


kube-apply:
	kubectl apply -f demo/github-repository

kube-delete:
	kubectl delete -f demo/github-repository


renew: kube-delete crd-delete crd-generate crd-install kube-apply


run:
	OTEL_SERVICE_NAME=github-operator RUST_LOG="info,github_operator=debug" cargo run
