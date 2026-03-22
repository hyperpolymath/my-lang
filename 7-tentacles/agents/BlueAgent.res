// BlueAgent.res - The Auditor
// Teaches: Verification, auditing, tracing, debugging, proof systems

open Types

let names: agentNames = {
  cuttle: "Detective Blue",
  squidlet: "Tracker Blue",
  duet: "Verification Agent Blue",
  octopus: "Verification Oracle Blue",
}

let personality: personality = {
  voice: "Curious and analytical, always asking questions and looking for clues",
  catchphrase: "The evidence never lies!",
  encouragement: [
    "Excellent deduction!",
    "You found a crucial clue!",
    "Your logic is impeccable!",
    "The mystery is unraveling!",
  ],
  corrections: [
    "Hmm, let's re-examine the evidence...",
    "That clue might mean something else.",
    "Good theory, but let's verify it.",
    "The facts don't quite add up yet.",
  ],
  celebrations: [
    "Case solved! Brilliant detective work!",
    "You've cracked the code!",
    "Irrefutable proof! Amazing!",
    "The mystery is completely solved!",
  ],
}

let teaches = [
  "Logical deduction",
  "Evidence gathering",
  "Proof construction",
  "Debugging techniques",
  "Execution tracing",
  "Hoare logic",
  "Formal verification",
  "Theorem proving",
]

// Get the agent's name for a given stage
let getName = (stage: stage): string => {
  switch stage {
  | Cuttle => names.cuttle
  | Squidlet => names.squidlet
  | Duet => names.duet
  | Octopus => names.octopus
  }
}

// Get what the agent is "secretly" teaching at each stage
let getHiddenConcept = (stage: stage): string => {
  switch stage {
  | Cuttle => "Logical deduction and proof"
  | Squidlet => "Logging and execution tracing"
  | Duet => "Formal verification and Hoare logic"
  | Octopus => "Theorem proving and certified systems"
  }
}

// Get a random encouragement message
let encourage = (): string => {
  let idx = Js.Math.random_int(0, Array.length(personality.encouragement))
  personality.encouragement[idx]->Option.getOr(personality.catchphrase)
}

// Get a random correction message
let correct = (): string => {
  let idx = Js.Math.random_int(0, Array.length(personality.corrections))
  personality.corrections[idx]->Option.getOr("Let's try again!")
}

// Get a random celebration message
let celebrate = (): string => {
  let idx = Js.Math.random_int(0, Array.length(personality.celebrations))
  personality.celebrations[idx]->Option.getOr(personality.catchphrase)
}

// Lesson definitions from curriculum/cuttle/blue/
let lessons: array<lesson> = [
  {
    id: "cuttle-blue-01",
    title: "Mystery Intro",
    agent: Blue,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Solve your first mystery using clues and logical deduction",
    objectives: [
      "How to gather clues",
      "How to use clues to figure things out",
      "How evidence points to answers",
    ],
    activities: [
      {
        activityId: "blue-01-gather-clues",
        activityType: Game({
          gameName: "Clue Finding",
          rules: ["Examine the scene carefully", "Click on suspicious items", "Record each clue you find"],
          winCondition: "Find all 3 clues",
        }),
        instructions: "The cookie jar is EMPTY! Examine the scene for clues. Look for crumbs, lid position, and chocolate stains.",
        hints: ["Look at the floor", "Check the dog bed", "Look at Buddy's nose"],
      },
      {
        activityId: "blue-01-interpret-clues",
        activityType: Puzzle({
          puzzleType: "deduction",
          pieces: 3,
          solution: "Crumbs lead to dog bed = dog carried cookie",
        }),
        instructions: "The crumbs lead to the dog bed. What does this mean?",
        hints: ["Think about who would go to the dog bed", "Crumbs show where someone walked"],
      },
      {
        activityId: "blue-01-solve-mystery",
        activityType: Puzzle({
          puzzleType: "conclusion",
          pieces: 3,
          solution: "Buddy the Dog",
        }),
        instructions: "Based on ALL the clues (crumbs to dog bed, chocolate on nose), who ate the last cookie?",
        hints: ["Combine all the evidence", "Who do all clues point to?"],
      },
    ],
    hiddenConcept: "Logical deduction basics",
    revealedConcept: None,
  },
]

// Create the complete agent definition
let agent: agent = {
  color: Blue,
  names,
  compilerRole: "Auditor - Verifies correctness and provides formal proofs",
  teaches,
  personality,
  lessons,
}

// Reveal text shown when transitioning stages
let revealText = (fromStage: stage, toStage: stage): option<string> => {
  switch (fromStage, toStage) {
  | (Cuttle, Squidlet) =>
    Some("All those mysteries you solved? Blue was teaching you about VERIFICATION! Every clue you found was like evidence that proves code is correct.")
  | (Squidlet, Duet) =>
    Some("Blue has been teaching you FORMAL VERIFICATION! When you proved who did it, you were learning how mathematicians prove that code can never fail.")
  | (Duet, Octopus) =>
    Some("You understand verification systems now! You know how Blue can mathematically PROVE that code is correct, not just test it.")
  | _ => None
  }
}
