---
"repo-roller-frontend": minor
---

The create-repository wizard now shows you more, in more places, before you commit to creating a
repository.

**Template details, without losing your place**: Step 1 (Choose a template) now shows a details
panel alongside the template grid. Selecting a card automatically loads and displays that
template's full description, every tag, its repository type (when set), and its full list of
variables — all without navigating away from the grid or advancing to a later step. On larger
screens the panel sits beside the grid; on narrower screens it moves below it.

**A wider Step 1**: to make room for the new details panel, Step 1 of the wizard — and the page
around it — now uses significantly more of the available width on large screens. The other steps
(Settings, Variables, Review) are unchanged and stay at their original width.

**New: Review and create step**: the wizard now always ends on a new "Review and create" step,
shown after Settings (or after Variables, if the template has any) and before the repository is
actually created. It shows the organisation/repository name, the selected template, its type and
visibility, the team you chose, and — new — every template variable you configured, with the
value you entered for each. Previously, configured variable values were used to create the
repository but were never shown back to you anywhere in the wizard; now you can check everything
in one place before committing.

**Create has moved**: the **Create Repository** button now appears exclusively on the new Review
step. Settings and Variables no longer create the repository directly — their buttons now always
read "Next: …" and simply advance the wizard. Clicking **← Back** from Review returns you to
whichever step preceded it, with every value you entered preserved.
