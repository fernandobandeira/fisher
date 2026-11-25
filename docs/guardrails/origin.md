# Where the ideas came from

This page describes where the ideas for independent expected values and the `Guard-Change:` line came from.

Before fisher, the owner researched how people review code written by
agents. Two ideas carried over. Someone other than the implementer
supplies the expected outcome before the code exists. The owner reviews
changes to the things that decide what passes.

StrongDM describes how an agent can rewrite a test to match its code or
make the code trivially pass. It says its scenarios often live outside the
codebase, like a holdout set. [StrongDM Software Factory](https://factory.strongdm.ai/)

Fisher keeps its values visible in the repository. It relies on committing
them first, keeping their authorship separate from the implementation, and
having the owner review changes. The first commit, `7a57941` on
2026-11-22, introduced `expectations/`, the `Guard-Change:` line, the
protected-path list and the hook. Those names and the hook were designed
for fisher during that setup.
