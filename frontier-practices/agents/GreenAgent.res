// GreenAgent.res - The AST Architect
// Teaches: Abstract syntax trees, code representation, manipulation

open Types

let names: agentNames = {
  cuttle: "Builder Green",
  squidlet: "Maker Green",
  duet: "Structure Agent Green",
  octopus: "Code Architecture Specialist Green",
}

let personality: personality = {
  voice: "Creative and constructive, always excited about building new things",
  catchphrase: "Let's build something amazing!",
  encouragement: [
    "Great foundation!",
    "That structure is solid!",
    "You're building beautifully!",
    "Perfect piece placement!",
  ],
  corrections: [
    "Hmm, that piece doesn't connect there...",
    "Let's check the blueprint again.",
    "The structure needs a stronger base.",
    "Almost! But buildings need foundations first.",
  ],
  celebrations: [
    "What an amazing creation!",
    "Architectural masterpiece!",
    "You built something incredible!",
    "That structure will stand forever!",
  ],
}

let teaches = [
  "Composition and construction",
  "Hierarchical thinking",
  "Tree structures",
  "Data representation",
  "Abstract syntax trees",
  "Intermediate representations",
  "Code generation",
  "Optimization passes",
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
  | Cuttle => "Composition and hierarchy"
  | Squidlet => "Tree structures and data representation"
  | Duet => "AST construction and manipulation"
  | Octopus => "Compiler IR and code generation"
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

// Lesson definitions from curriculum/cuttle/green/
let lessons: array<lesson> = [
  {
    id: "cuttle-green-01",
    title: "Building Blocks",
    agent: Green,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Learn the basics of construction - how small parts make bigger things",
    objectives: [
      "How small parts make bigger things",
      "How to stack blocks",
      "How to plan what you build",
    ],
    activities: [
      {
        activityId: "green-01-stack-blocks",
        activityType: Game({
          gameName: "Block Stacking",
          rules: ["Stack three blocks vertically", "Each block goes on top of the previous", "Build upward"],
          winCondition: "Create a stable 3-block tower",
        }),
        instructions: "Let's start simple. Stack three blocks to make a tower!",
        hints: ["Start at the bottom", "Place each block carefully"],
      },
      {
        activityId: "green-01-build-house",
        activityType: Game({
          gameName: "House Assembly",
          rules: ["Place the base first", "Add walls on the base", "Add the roof on top"],
          winCondition: "Complete house with base, walls, and roof",
        }),
        instructions: "A house has parts: a base, walls, and a roof. Build one!",
        hints: ["Base goes at the bottom", "Walls go in the middle", "Roof goes on top"],
      },
      {
        activityId: "green-01-build-order",
        activityType: Puzzle({
          puzzleType: "ordering",
          pieces: 3,
          solution: "Bottom (base) first, then middle, then top",
        }),
        instructions: "If we're building a tower, what order do we need? What do you build FIRST?",
        hints: ["Think about gravity", "You can't build the roof first!"],
      },
    ],
    hiddenConcept: "Composition and basic structure",
    revealedConcept: None,
  },
]

// Create the complete agent definition
let agent: agent = {
  color: Green,
  names,
  compilerRole: "AST Architect - Builds and transforms code representations",
  teaches,
  personality,
  lessons,
}

// Reveal text shown when transitioning stages
let revealText = (fromStage: stage, toStage: stage): option<string> => {
  switch (fromStage, toStage) {
  | (Cuttle, Squidlet) =>
    Some("All those building blocks? Green was teaching you about STRUCTURE! Every tower you built was like a tree of code, with branches and leaves.")
  | (Squidlet, Duet) =>
    Some("Green has been teaching you about ABSTRACT SYNTAX TREES! When you assembled pieces into complex structures, you were learning how compilers represent code internally.")
  | (Duet, Octopus) =>
    Some("You understand code architecture now! You know how Green transforms human-readable code into tree structures that computers can execute.")
  | _ => None
  }
}
