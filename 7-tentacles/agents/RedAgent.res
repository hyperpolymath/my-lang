// RedAgent.res - The Parser
// Teaches: Lexical analysis → Parsing → Syntax trees

open Types

let names: agentNames = {
  cuttle: "Speedy Red",
  squidlet: "Fast Finder Red",
  duet: "Performance Agent Red",
  octopus: "Performance Agent Red",
}

let personality: personality = {
  voice: "Energetic and fast-talking, always excited about speed and efficiency",
  catchphrase: "Let's zoom through this!",
  encouragement: [
    "You're getting faster!",
    "That pattern was perfect!",
    "Speedy work!",
    "You're thinking like a racer now!",
  ],
  corrections: [
    "Hmm, let's try a different path!",
    "Almost! What if we went faster here?",
    "Good try! Speed isn't just about going fast...",
    "Let's look at this pattern again!",
  ],
  celebrations: [
    "ZOOM! You did it!",
    "New record! Amazing!",
    "That was lightning fast!",
    "You've mastered this track!",
  ],
}

let teaches = [
  "Pattern recognition",
  "Algorithmic thinking",
  "Efficiency and optimization",
  "Lexical analysis",
  "Recursive descent parsing",
  "Grammar and syntax rules",
  "Tokenization",
  "Abstract syntax tree construction",
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
  | Cuttle => "Pattern recognition and rule-following"
  | Squidlet => "Algorithmic complexity and optimization"
  | Duet => "Lexical analysis and parsing"
  | Octopus => "Complete parser implementation"
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

// Lesson definitions from curriculum/cuttle/red/
let lessons: array<lesson> = [
  {
    id: "cuttle-red-01",
    title: "Racing Intro",
    agent: Red,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Learn the basics of racing - following paths and recognizing simple patterns",
    objectives: [
      "How to follow a path",
      "How to go fast (but not TOO fast!)",
      "How to recognize simple patterns",
    ],
    activities: [
      {
        activityId: "red-01-follow-path",
        activityType: Game({
          gameName: "Follow the Path",
          rules: ["Trace the path from START to FINISH", "Follow the arrows: RIGHT → RIGHT → DOWN → FINISH"],
          winCondition: "Successfully trace the complete path",
        }),
        instructions: "See this track? Trace the path with your finger from START to FINISH!",
        hints: ["Start at START", "Follow each arrow in order"],
      },
      {
        activityId: "red-01-race-time",
        activityType: Game({
          gameName: "Drag Race",
          rules: ["Drag Speedy Red along the track", "Stay on the path"],
          winCondition: "Reach the finish line",
        }),
        instructions: "Now let's race! Drag Speedy Red along the track.",
        hints: ["Click and drag to move", "Follow the track carefully"],
      },
      {
        activityId: "red-01-faster-path",
        activityType: Puzzle({
          puzzleType: "multiple-choice",
          pieces: 2,
          solution: "Path A (3 turns)",
        }),
        instructions: "Look at two paths. Which one is faster? Path A has 3 turns, Path B has 5 turns.",
        hints: ["Count the turns", "Fewer turns usually means faster"],
      },
    ],
    hiddenConcept: "Pattern recognition and rule-following",
    revealedConcept: None,
  },
  {
    id: "cuttle-red-02",
    title: "Different Tracks",
    agent: Red,
    stage: Cuttle,
    difficulty: Introductory,
    description: "Race on different types of tracks and learn different strategies",
    objectives: [
      "Different tracks need different strategies",
      "Some tracks are straight, some are twisty",
      "Choosing the right approach matters",
    ],
    activities: [
      {
        activityId: "red-02-straight-track",
        activityType: Game({
          gameName: "Straight Track Race",
          rules: ["Race on a straight track", "Go as fast as possible"],
          winCondition: "Achieve the fastest time",
        }),
        instructions: "This track goes in ONE direction. Nice and simple - go FULL SPEED!",
        hints: ["No turns needed", "Just go straight"],
      },
      {
        activityId: "red-02-zigzag-track",
        activityType: Game({
          gameName: "Zigzag Track Race",
          rules: ["Navigate the zigzag path", "Slow down at corners"],
          winCondition: "Complete without hitting walls",
        }),
        instructions: "This track goes back and forth. You need to change direction!",
        hints: ["Slow down at corners", "Quick turns are key"],
      },
      {
        activityId: "red-02-spiral-track",
        activityType: Game({
          gameName: "Spiral Track Race",
          rules: ["Follow the spiral inward", "Path gets tighter as you go"],
          winCondition: "Reach the center",
        }),
        instructions: "This track goes round and round, getting smaller!",
        hints: ["Start fast", "Get slower and more careful as you approach the center"],
      },
      {
        activityId: "red-02-match-strategy",
        activityType: Puzzle({
          puzzleType: "matching",
          pieces: 3,
          solution: "Straight=Full speed, Zigzag=Slow at turns, Spiral=Start fast then careful",
        }),
        instructions: "Match each track with the best racing strategy.",
        hints: ["Think about how each track shape affects speed"],
      },
    ],
    hiddenConcept: "Different algorithms for different problems",
    revealedConcept: None,
  },
  {
    id: "cuttle-red-03",
    title: "Obstacle Courses",
    agent: Red,
    stage: Cuttle,
    difficulty: Beginner,
    description: "Learn to navigate obstacle courses and spot patterns",
    objectives: [
      "How to recognize obstacles",
      "How to choose a different path when blocked",
      "How to spot patterns in obstacle placement",
    ],
    activities: [
      {
        activityId: "red-03-spot-obstacle",
        activityType: Puzzle({
          puzzleType: "identification",
          pieces: 3,
          solution: "Rock blocks, Puddle slows, Boost speeds",
        }),
        instructions: "Learn the obstacles: Rock 🪨 blocks you, Puddle 💧 slows you, Boost ⚡ speeds you!",
        hints: ["Rocks cannot be passed through", "Puddles are slow but possible", "Boosts help you"],
      },
      {
        activityId: "red-03-navigate-course",
        activityType: Game({
          gameName: "Obstacle Navigation",
          rules: ["Go around rocks", "Avoid or accept puddle slowdown", "Hit boosts for speed"],
          winCondition: "Fastest time through the course",
        }),
        instructions: "Navigate the course: go around rocks, through or around puddles, and HIT those boosts!",
        hints: ["Plan your route first", "Boosts save time"],
      },
      {
        activityId: "red-03-pattern-spotting",
        activityType: Puzzle({
          puzzleType: "sequence",
          pieces: 8,
          solution: "Boost (⚡)",
        }),
        instructions: "The obstacles follow a PATTERN! Rock, Puddle, Boost, Rock, Puddle, Boost... What comes next?",
        hints: ["Look for the repeating sequence", "The pattern has 3 items"],
      },
      {
        activityId: "red-03-big-course",
        activityType: Challenge({
          challengeType: "complex-obstacle-course",
          timeLimit: Some(120),
          scoring: "Time-based with pattern bonus",
        }),
        instructions: "Race through a course with MANY obstacles! Use pattern recognition!",
        hints: ["Look ahead for patterns", "Plan your boosts strategically"],
      },
    ],
    hiddenConcept: "Pattern matching and conditional logic",
    revealedConcept: None,
  },
  {
    id: "cuttle-red-04",
    title: "Speed Measurements",
    agent: Red,
    stage: Cuttle,
    difficulty: Beginner,
    description: "Learn to measure and compare efficiency through step counting",
    objectives: [
      "How to count steps",
      "How to compare different paths",
      "Why counting steps matters",
    ],
    activities: [
      {
        activityId: "red-04-counting-steps",
        activityType: Puzzle({
          puzzleType: "comparison",
          pieces: 2,
          solution: "Path A (3 steps)",
        }),
        instructions: "Count the steps in each path. Path A: 3 steps. Path B: 5 steps. Which is fewer?",
        hints: ["Fewer steps = faster finish"],
      },
      {
        activityId: "red-04-step-counter-race",
        activityType: Game({
          gameName: "Step Counter Race",
          rules: ["Race through branching paths", "Watch the step counter", "Minimize total steps"],
          winCondition: "Finish with the fewest steps",
        }),
        instructions: "Race through the course. Watch the step counter - can you do it in fewer steps?",
        hints: ["Some paths look longer but have fewer steps", "Experiment to find the best route"],
      },
      {
        activityId: "red-04-compare-paths",
        activityType: Puzzle({
          puzzleType: "path-comparison",
          pieces: 2,
          solution: "Path A (6 steps) is more efficient than Path B (8 steps)",
        }),
        instructions: "Compare two paths to the same finish. Count the steps for each!",
        hints: ["Count each move as one step", "Include turns in your count"],
      },
      {
        activityId: "red-04-scaling-challenge",
        activityType: Challenge({
          challengeType: "scaling-comparison",
          timeLimit: None,
          scoring: "Understanding of growth patterns",
        }),
        instructions: "Watch what happens as the track gets bigger! Some approaches don't SCALE!",
        hints: ["Watch the step graph", "Notice how zigzag gets worse faster"],
      },
    ],
    hiddenConcept: "Algorithmic complexity and efficiency measurement",
    revealedConcept: None,
  },
  {
    id: "cuttle-red-05",
    title: "Racing Rules",
    agent: Red,
    stage: Cuttle,
    difficulty: Beginner,
    description: "Learn about valid moves, rule checking, and validation",
    objectives: [
      "What a \"valid move\" is",
      "What happens when you break the rules",
      "How to check if a move is okay",
    ],
    activities: [
      {
        activityId: "red-05-valid-invalid",
        activityType: Puzzle({
          puzzleType: "validation",
          pieces: 4,
          solution: "Invalid - crossing grass is not allowed",
        }),
        instructions: "Is this move allowed? Check if it crosses grass (not allowed) or stays on track (allowed).",
        hints: ["Stay on the track (⬜)", "Grass (🟩) is off-limits"],
      },
      {
        activityId: "red-05-rule-checker",
        activityType: Game({
          gameName: "Rule Checker",
          rules: ["Stay on track", "Move one square at a time", "No diagonal moves", "No teleporting"],
          winCondition: "Complete course with only valid moves",
        }),
        instructions: "Try making moves. The Rule Checker will tell you if they're valid!",
        hints: ["Watch for the red X - that means invalid", "Only move up, down, left, or right"],
      },
      {
        activityId: "red-05-fix-path",
        activityType: Puzzle({
          puzzleType: "error-correction",
          pieces: 5,
          solution: "Fix the diagonal move at position [2,1]",
        }),
        instructions: "This path has MISTAKES. Find and fix the invalid moves!",
        hints: ["Look for diagonal moves", "Check each step against the rules"],
      },
      {
        activityId: "red-05-write-valid-path",
        activityType: Creative({
          medium: "path-building",
          prompt: "Write a valid path from [0,0] to [3,3] that follows all the rules",
        }),
        instructions: "Now YOU write a path that follows ALL the rules!",
        hints: ["Use only up/down/left/right", "Avoid obstacles", "No skipping squares"],
      },
    ],
    hiddenConcept: "Grammar and syntax rules",
    revealedConcept: None,
  },
]

// Create the complete agent definition
let agent: agent = {
  color: Red,
  names,
  compilerRole: "Parser - Transforms source code into structured syntax trees",
  teaches,
  personality,
  lessons,
}

// Reveal text shown when transitioning stages
let revealText = (fromStage: stage, toStage: stage): option<string> => {
  switch (fromStage, toStage) {
  | (Cuttle, Squidlet) =>
    Some("Remember all those racing games? Red wasn't just teaching you to go fast - Red was teaching you to find PATTERNS. Every race track was like a sentence, and you learned to read them!")
  | (Squidlet, Duet) =>
    Some("Red has been teaching you PARSING all along! When you found the fastest path through obstacles, you were learning how compilers break down code into pieces they can understand.")
  | (Duet, Octopus) =>
    Some("You've mastered what Red teaches: lexical analysis and parsing. You can now build the first stages of a real compiler!")
  | _ => None
  }
}
