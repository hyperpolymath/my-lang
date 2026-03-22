// IndigoAgent.res - The Compile-Time Metaprogrammer
// Teaches: Metaprogramming, compile-time evaluation, macros, reflection

open Types

let names: agentNames = {
  cuttle: "Magic Indigo",
  squidlet: "Spell Indigo",
  duet: "Metaprogramming Wizard Indigo",
  octopus: "Compile-Time Execution Master Indigo",
}

let personality: personality = {
  voice: "Mysterious and whimsical, speaking in riddles that reveal deep truths",
  catchphrase: "The real magic happens before the show begins!",
  encouragement: [
    "Your spell is taking shape!",
    "The magic flows through you!",
    "What a powerful incantation!",
    "You're learning the ancient ways!",
  ],
  corrections: [
    "The spell fizzled... let's adjust the formula.",
    "Magic requires precise ingredients.",
    "Almost! But the timing was off.",
    "That spell needs more preparation.",
  ],
  celebrations: [
    "MAGNIFICENT MAGIC!",
    "You've mastered the arcane arts!",
    "A spell for the ages!",
    "True wizardry! Extraordinary!",
  ],
}

let teaches = [
  "Transformation and pattern rules",
  "Abstraction and shortcuts",
  "Template systems",
  "Macro programming",
  "Compile-time computation",
  "Staged computation",
  "Partial evaluation",
  "Code generation",
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
  | Cuttle => "Transformation and rule-based patterns"
  | Squidlet => "Macros and templating"
  | Duet => "Compile-time evaluation and staging"
  | Octopus => "Partial evaluation and supercompilation"
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

// Lesson definitions from curriculum/cuttle/indigo/
let lessons: array<lesson> = [
  {
    id: "cuttle-indigo-01",
    title: "Magic Show",
    agent: Indigo,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Learn how magic works through transformation rules - predictable spells!",
    objectives: [
      "How transformations work",
      "Magic follows rules",
      "The same spell always does the same thing",
    ],
    activities: [
      {
        activityId: "indigo-01-color-swap",
        activityType: Game({
          gameName: "Color Swap Spell",
          rules: ["The spell transforms red to blue", "Cast the spell on red objects", "Watch the transformation"],
          winCondition: "Successfully transform 3 red objects to blue",
        }),
        instructions: "This spell changes colors! Red becomes Blue. Try casting the Color Swap spell!",
        hints: ["Click on red objects", "Watch them change to blue"],
      },
      {
        activityId: "indigo-01-grow-spell",
        activityType: Game({
          gameName: "Growing Spell",
          rules: ["The grow spell makes things bigger", "Cast on any object", "Object doubles in size"],
          winCondition: "Grow 3 objects",
        }),
        instructions: "This spell makes things BIGGER! Cast the Grow spell on small objects.",
        hints: ["Small objects become big", "The spell ALWAYS makes things bigger"],
      },
      {
        activityId: "indigo-01-predict-magic",
        activityType: Puzzle({
          puzzleType: "prediction",
          pieces: 3,
          solution: "Blue (becomes blue)",
        }),
        instructions: "If the Color Swap spell turns RED into BLUE... what will a red ball become?",
        hints: ["The spell is predictable", "Red always becomes blue"],
      },
      {
        activityId: "indigo-01-chain-spells",
        activityType: Game({
          gameName: "Spell Chaining",
          rules: ["Apply Color Swap first", "Then apply Grow", "Chain creates combined effect"],
          winCondition: "Create a big blue ball from a small red ball",
        }),
        instructions: "Chain TWO spells: Color Swap then Grow. Small red ball becomes big blue ball!",
        hints: ["Order matters", "First change color, then change size"],
      },
    ],
    hiddenConcept: "Basic transformation rules",
    revealedConcept: None,
  },
]

// Create the complete agent definition
let agent: agent = {
  color: Indigo,
  names,
  compilerRole: "Metaprogrammer - Executes code at compile time to generate optimized code",
  teaches,
  personality,
  lessons,
}

// Reveal text shown when transitioning stages
let revealText = (fromStage: stage, toStage: stage): option<string> => {
  switch (fromStage, toStage) {
  | (Cuttle, Squidlet) =>
    Some("All that magic? Indigo was teaching you about TRANSFORMATION! Every spell you cast was like a program that writes other programs.")
  | (Squidlet, Duet) =>
    Some("Indigo has been teaching you METAPROGRAMMING! When you cast spells that created new spells, you were learning how code can generate code.")
  | (Duet, Octopus) =>
    Some("You understand compile-time execution now! You know how Indigo runs code BEFORE the program runs, creating specialized, optimized programs.")
  | _ => None
  }
}
