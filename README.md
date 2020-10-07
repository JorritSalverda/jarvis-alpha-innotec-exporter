## Installation

To install this application using Helm run the following commands: 

```bash
helm repo add jorritsalverda https://helm.jorritsalverda.com
kubectl create namespace jarvis-alpha-innotec-exporter

helm upgrade \
  jarvis-alpha-innotec-exporter \
  jorritsalverda/jarvis-alpha-innotec-exporter \
  --install \
  --namespace jarvis-alpha-innotec-exporter \
  --set secret.gcpServiceAccountKeyfile='{abc: blabla}' \
  --wait
```
