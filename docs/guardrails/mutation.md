# Mutation testing

This page explains why the pull-request mutation job tests the whole workspace.

The pull-request mutation job runs the whole workspace. Diff scoping was removed because changes to shared tests, manifests, expected data and the gate itself can affect mutants outside the diff. Reintroduce scoping if the hosted run grows past about 15 minutes.
