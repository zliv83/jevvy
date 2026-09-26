# Vision: give your software a sense of judgment

> The pitch, in plain words. `ROADMAP2.md` turns each paragraph below into a piece of the crate.

Every program you've ever used is deaf and blind to meaning. It knows a button was clicked, a form was submitted, a message arrived. It has no idea whether the customer is angry, whether the two orders are really the same order, whether this refund request is the kind your policy covers. For thirty years the only fix was a human reading the screen. For the last three, the fix has been to bolt on a chatbot and hope it says something useful.

Jev is the third option. It is not a brain that talks. It is a thousand tiny judges that answer instantly, in parallel, and never say more than you asked. You write the question in plain English, you write the possible answers, and Jev hands back how likely each one is. That's it. It cannot invent an option you didn't offer. It cannot wander off. It answers in about the time a database does, for a fraction of a cent.

Here is what software built on it feels like.

**It notices.** Instead of waiting for someone to complain, the program has standing questions it asks of its own state every time something changes: is this thread going off the rails, does this invoice look like a duplicate, is this new user about to give up? Think of them as smoke detectors for meaning. They're cheap, so you install hundreds. The program stops being a filing cabinet and starts being a colleague who glances over your shoulder.

**It moves only when something changes.** Jev has no memory and no mood. Ask it the same question about the same moment a hundred times and you get the same answer, so there is no reason to ask twice until the world moves. The program walks its own state like a reader walking a choose-your-own-adventure book: every page and every option was written by you, Jev picks which way to turn, and your code decides whether it's sure enough to turn the page or should hand the book to a person.

**It knows what it doesn't know.** Every answer comes with how confident it is, and that number is honest. Sure things happen automatically. Unsure things go to a human. The system has an "I'm not certain" door built into the front of it, which is the thing every automated system before this one was missing.

**It keeps receipts, and it can replay.** Because the answer is a probability over options you wrote, every automated decision comes with the question, the choices, and the numbers behind it. And because Jev is consistent, you can take a new question and run it across every moment your program ever saw. Want to know how a new refund rule would have played out over last year's tickets? Ask, and have the answer in minutes, with no one's inbox involved.

Chatbots made computers talk. Jev makes them notice, decide, admit doubt, and show their work. That's what it takes to let software run a piece of your business without a human watching every step.

---

## Who writes what

One correction that keeps the picture honest, because it decides the shape of the crate:

- **Code and data write the book.** Every page, every question, every option. Some pages are typed out by hand; some are stamped out of data (one question per candidate record, options from a taxonomy branch). Either way, your program authored them.
- **Jev reads the book.** At each fork it picks an option, places a position on a scale, or says how likely a yes is, and reports how sure it is.
- **Code holds the guardrails.** Thresholds, weights, what "unsure" means for this action, when a human steps in. Jev never adds a page and never chooses its own next question.

"Ask everything at once" applies to one fork, not to the whole book. Every question about the *same moment* goes in one request. A new moment is a new request. A long-running program calls Jev as often as its state changes, and no more.
