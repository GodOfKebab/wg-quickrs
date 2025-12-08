# Manual Deployment Testing with Ansible/Terraform

Aside from the automated tests and the manual vagrant box testing, we can also test the manual deployment process on cloud providers to optimize for hassle-free setup.

Requirements:
* Ansible
* Terraform

## AWS

To set up/destroy resources on AWS, run the following ansible playbooks:

```shell
# on the host machine at wg-quickrs/tests/deployment/aws
# to DEPLOY the resources run:
ansible-playbook deploy.yml
# to DESTROY the resources run:
# ansible-playbook destroy.yml
```
