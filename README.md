## Installation

To install this application using Helm run the following commands: 

```bash
helm repo add jorritsalverda https://helm.jorritsalverda.com
kubectl create namespace alpha-innotec-bigquery-exporter

helm upgrade \
  alpha-innotec-bigquery-exporter \
  jorritsalverda/alpha-innotec-bigquery-exporter \
  --install \
  --namespace alpha-innotec-bigquery-exporter \
  --set secret.gcpServiceAccountKeyfile='{abc: blabla}' \
  --wait
```
