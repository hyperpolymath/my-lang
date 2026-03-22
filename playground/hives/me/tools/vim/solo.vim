" Vim syntax file for My Language (Solo/Duet/Ensemble)

if exists("b:current_syntax")
  finish
endif

" Keywords
syn keyword soloKeyword fn let mut const static if else match loop while for in return break continue
syn keyword soloKeyword struct enum trait impl mod use pub type where as ref move unsafe
syn keyword soloKeyword async await affine comptime
syn keyword soloKeyword pre post invariant requires ensures
syn keyword soloKeyword intent hybrid synth verify
syn keyword soloKeyword agent workflow spawn send receive broadcast state capabilities goals

" Types
syn keyword soloType i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize f32 f64 bool char str

" Constants
syn keyword soloBoolean true false

" Comments
syn region soloLineComment start="//" end="$"
syn region soloBlockComment start="/\*" end="\*/"

" Strings
syn region soloString start='"' end='"' skip='\\"'
syn region soloChar start="'" end="'"

" Numbers
syn match soloNumber "\<\d\+\>"
syn match soloFloat "\<\d\+\.\d\+\>"

" Attributes
syn match soloAttribute "#\[.\{-}\]"
syn match soloAnnotation "@\w\+"

" Highlighting
hi def link soloKeyword Keyword
hi def link soloType Type
hi def link soloBoolean Boolean
hi def link soloLineComment Comment
hi def link soloBlockComment Comment
hi def link soloString String
hi def link soloChar Character
hi def link soloNumber Number
hi def link soloFloat Float
hi def link soloAttribute PreProc
hi def link soloAnnotation Special

let b:current_syntax = "solo"
