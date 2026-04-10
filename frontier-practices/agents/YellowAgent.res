// YellowAgent.res - The Type System
// Teaches: Type systems, affine types, memory safety, ownership

open Types

let names: agentNames = {
  cuttle: "Safety Yellow",
  squidlet: "Checker Yellow",
  duet: "Type System Yellow",
  octopus: "Safety Guarantor Yellow",
}

let personality: personality = {
  voice: "Careful and methodical, always making sure things are in the right place",
  catchphrase: "Everything has its place!",
  encouragement: [
    "Perfect classification!",
    "You found exactly the right spot!",
    "That's the correct type!",
    "You're keeping everything organized!",
  ],
  corrections: [
    "Hmm, that doesn't quite fit there...",
    "Let's check what type this is again.",
    "Almost! But this belongs somewhere else.",
    "The shapes don't match - let's look closer.",
  ],
  celebrations: [
    "Everything is in perfect order!",
    "Type-safe perfection!",
    "Not a single thing out of place!",
    "You've organized it all beautifully!",
  ],
}

let teaches = [
  "Classification and categorization",
  "Rules and constraints",
  "Logical thinking",
  "Type checking",
  "Affine types and ownership",
  "Linear logic",
  "Memory safety",
  "Formal verification basics",
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
  | Cuttle => "Classification and organization"
  | Squidlet => "Type checking and contracts"
  | Duet => "Type inference and affine types"
  | Octopus => "Formal type systems and proofs"
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

// Lesson definitions from curriculum/cuttle/yellow/
let lessons: array<lesson> = [
  {
    id: "cuttle-yellow-01",
    title: "Color Sorting",
    agent: Yellow,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Learn to sort and organize things by color - the basics of classification",
    objectives: [
      "How to identify colors",
      "How to sort into groups",
      "Why organization matters",
    ],
    activities: [
      {
        activityId: "yellow-01-color-matching",
        activityType: Puzzle({
          puzzleType: "matching",
          pieces: 4,
          solution: "Apple=Red, Banana=Yellow, Blueberry=Blue, Lettuce=Green",
        }),
        instructions: "Look at these items. What color is each one? Match them!",
        hints: ["Look at each item carefully", "Think about the color you see"],
      },
      {
        activityId: "yellow-01-sort-shapes",
        activityType: Game({
          gameName: "Sorting Game",
          rules: ["Drag each shape to the correct color bin", "Red shapes go in Red Bin", "Blue shapes go in Blue Bin"],
          winCondition: "All shapes correctly sorted",
        }),
        instructions: "Drag each shape to the correct color bin!",
        hints: ["Look at the color of each shape", "Match it to the bin with the same color"],
      },
      {
        activityId: "yellow-01-doesnt-belong",
        activityType: Puzzle({
          puzzleType: "odd-one-out",
          pieces: 4,
          solution: "Lemon (yellow, not red)",
        }),
        instructions: "One of these is NOT like the others. Which item doesn't belong in the RED group?",
        hints: ["Look for the item that's a different color", "The lemon stands out"],
      },
    ],
    hiddenConcept: "Basic type classification",
    revealedConcept: None,
  },
]

// Create the complete agent definition
let agent: agent = {
  color: Yellow,
  names,
  compilerRole: "Type System - Ensures type safety and prevents errors at compile time",
  teaches,
  personality,
  lessons,
}

// Reveal text shown when transitioning stages
let revealText = (fromStage: stage, toStage: stage): option<string> => {
  switch (fromStage, toStage) {
  | (Cuttle, Squidlet) =>
    Some("All that sorting and organizing? Yellow was teaching you about TYPES! Every category you created was like a type in a programming language.")
  | (Squidlet, Duet) =>
    Some("Yellow has been teaching you TYPE SAFETY! When you made sure shapes fit in the right holes, you were learning how compilers prevent crashes and bugs.")
  | (Duet, Octopus) =>
    Some("You now understand type systems deeply! You know how Yellow checks that everything fits together, preventing entire categories of bugs before code even runs.")
  | _ => None
  }
}
