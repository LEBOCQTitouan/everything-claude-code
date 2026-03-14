.PHONY: ci ci-dry ci-job

## Run all CI workflows locally via act
ci:
	act

## Dry-run: list all jobs without executing
ci-dry:
	act -n -l

## Run a specific job: make ci-job JOB=validate
ci-job:
	act -j $(JOB)
