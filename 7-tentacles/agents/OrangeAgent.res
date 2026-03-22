// OrangeAgent.res - The Concurrency Engine
// Teaches: Async/await, scheduling, event loops, concurrency

open Types

let names: agentNames = {
  cuttle: "Juggler Orange",
  squidlet: "Event Orange",
  duet: "Concurrency Agent Orange",
  octopus: "Concurrency Orchestrator Orange",
}

let personality: personality = {
  voice: "Calm and rhythmic, always counting beats and keeping things in sync",
  catchphrase: "Keep all the balls in the air!",
  encouragement: [
    "Great timing!",
    "You're keeping everything in sync!",
    "Perfect rhythm!",
    "You're juggling like a pro!",
  ],
  corrections: [
    "Oops! One ball dropped. Let's try again!",
    "The timing was a bit off there...",
    "Remember: each ball needs its moment!",
    "Let's slow down and find the rhythm.",
  ],
  celebrations: [
    "Amazing coordination!",
    "You kept everything spinning perfectly!",
    "Master juggler achievement unlocked!",
    "Not a single drop! Incredible!",
  ],
}

let teaches = [
  "Coordination and timing",
  "Sequencing multiple tasks",
  "Event-driven thinking",
  "Queue management",
  "Async/await patterns",
  "Promise chains",
  "Race condition awareness",
  "Scheduler design",
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
  | Cuttle => "Timing and coordination basics"
  | Squidlet => "Event systems and queues"
  | Duet => "Async/await and promises"
  | Octopus => "Concurrent system architecture"
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

// Lesson definitions from curriculum/cuttle/orange/
let lessons: array<lesson> = [
  {
    id: "cuttle-orange-01",
    title: "Basic Juggling",
    agent: Orange,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Learn the basics of juggling - timing, rhythm, and coordination",
    objectives: [
      "How to catch and throw",
      "The rhythm of juggling",
      "Keeping track of timing",
    ],
    activities: [
      {
        activityId: "orange-01-one-ball",
        activityType: Game({
          gameName: "One Ball Juggle",
          rules: ["Throw the ball up", "Wait for it to come down", "Catch it", "Repeat the rhythm"],
          winCondition: "Maintain 10 successful throw-catch cycles",
        }),
        instructions: "Let's practice with ONE ball. Easy! Throw... catch... throw... catch!",
        hints: ["Watch the ball", "Keep a steady rhythm"],
      },
      {
        activityId: "orange-01-rhythm",
        activityType: Game({
          gameName: "Rhythm Clap",
          rules: ["Clap on beats 1 and 3", "Follow the tempo", "Stay in sync"],
          winCondition: "Complete 8 bars without missing a beat",
        }),
        instructions: "Juggling has a BEAT. Like music! THROW-wait-CATCH-wait. Can you clap along?",
        hints: ["Count: 1, 2, 3, 4", "Clap on 1 and 3"],
      },
      {
        activityId: "orange-01-timed-throws",
        activityType: Game({
          gameName: "Timed Throws",
          rules: ["Throw on beat 1", "Catch on beat 3", "Keep the timing steady"],
          winCondition: "10 perfectly timed throw-catch sequences",
        }),
        instructions: "Now throw the ball in TIME with the beat!",
        hints: ["The secret to juggling is rhythm", "Listen for the beat"],
      },
    ],
    hiddenConcept: "Sequential task execution",
    revealedConcept: None,
  },
]

// Create the complete agent definition
let agent: agent = {
  color: Orange,
  names,
  compilerRole: "Concurrency Engine - Manages parallel execution and scheduling",
  teaches,
  personality,
  lessons,
}

// Reveal text shown when transitioning stages
let revealText = (fromStage: stage, toStage: stage): option<string> => {
  switch (fromStage, toStage) {
  | (Cuttle, Squidlet) =>
    Some("All that juggling? Orange wasn't just teaching you to catch balls - Orange was teaching you to handle EVENTS. Each ball was like a task waiting for its turn!")
  | (Squidlet, Duet) =>
    Some("Orange has been teaching you CONCURRENCY! When you juggled multiple balls, you were learning how computers handle many tasks at once without dropping any.")
  | (Duet, Octopus) =>
    Some("You understand concurrent systems now! You know how Orange schedules which task runs when, preventing race conditions and keeping everything in harmony.")
  | _ => None
  }
}
