---
name: iac
category: cicd
tags: [cicd, infrastructure, terraform, declarative]
languages: [all]
difficulty: intermediate
---

## Intent

Manage infrastructure through machine-readable definition files rather than manual processes, enabling version control, review, and reproducibility.

## Problem

Manual infrastructure provisioning is slow, error-prone, and undocumented. Environments drift from each other because changes are applied ad hoc. Disaster recovery requires tribal knowledge. You need infrastructure that is auditable, repeatable, and reviewable.

## Solution

Define all infrastructure as code in declarative configuration files. Store them in version control alongside application code. Apply changes through automated pipelines with plan/apply cycles. Use state files to track what exists and detect drift.

## Language Implementations

### Terraform

```hcl
terraform {
  required_providers {
    aws = { source = "hashicorp/aws", version = "~> 5.0" }
  }
  backend "s3" {
    bucket = "my-terraform-state"
    key    = "prod/terraform.tfstate"
    region = "us-east-1"
  }
}

resource "aws_instance" "app" {
  ami           = var.ami_id
  instance_type = "t3.medium"

  tags = {
    Name        = "app-server"
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}

output "instance_ip" {
  value = aws_instance.app.public_ip
}
```

### GitHub Actions (Terraform Pipeline)

```yaml
jobs:
  plan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: hashicorp/setup-terraform@v3
      - run: terraform init
      - run: terraform plan -out=tfplan
      - uses: actions/upload-artifact@v4
        with:
          name: tfplan
          path: tfplan

  apply:
    needs: plan
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v4
      - uses: hashicorp/setup-terraform@v3
      - run: terraform init
      - uses: actions/download-artifact@v4
        with: { name: tfplan }
      - run: terraform apply tfplan
```

## When to Use

- When infrastructure must be reproducible across environments.
- When changes require audit trails and peer review.
- When disaster recovery must be automated.

## When NOT to Use

- When infrastructure is truly static and never changes.
- When the team lacks IaC expertise and the learning curve outweighs benefits.
- When prototyping disposable environments that will never reach production.

## Anti-Patterns

- Applying changes manually then importing into IaC state after the fact.
- Storing state files locally instead of in a shared, locked backend.
- Hardcoding environment-specific values instead of using variables.

## Related Patterns

- [cicd/gitops](gitops.md) — Git as the single source of truth for desired state.
- [cicd/pipeline-as-code](pipeline-as-code.md) — define CI/CD pipelines as versioned code.
- [cicd/trunk-based-dev](trunk-based-dev.md) — short-lived branches for infrastructure changes.

## References

- HashiCorp — Infrastructure as Code: https://www.hashicorp.com/resources/what-is-infrastructure-as-code
- Terraform Documentation: https://developer.hashicorp.com/terraform/docs
