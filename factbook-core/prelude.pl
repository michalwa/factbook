% Defines predicates available in the embedded Prolog runtime
% Required in order to build the initial Prolog state

:- op(50, fx, @). % `@T` represents a tag `T`
:- op(60, fx, ^). % `^T` represents "not T"

% `T!` represents "exactly T", i.e. a value `X` such that `X == T`. This may be
% useful if you rely on variables in your entries to represent empty values:
%
%   1: @task(_)
%   2: @task(x)
%
% `@task(x)` returns both entries 1 and 2, because `x` unifies with `_`
% `@task(x!)` returns only entry 2
%
:- op(60, xf, !).

% `_` succeeds for all entries. This is the first clause to ensure all
% subsequent clauses only match sufficiently instantiated views.
view_entry(_, V, _) :- var(V), !.

% `@P` succeeds for all entries containing tags which satisfy the pattern `P`.
% Patterns are simply unified with tags, unless they are compound terms
% containing special operators. See `pattern_arg/2` for details.
view_entry(C, @P, E) :-
  pattern_tag(P, Ps, T, Ts),
  entry_tag(C, E, T),
  maplist(pattern_arg, Ps, Ts).

view_entry(C, ^V, E)     :- entry(C, E), \+ view_entry(C, V, E).
view_entry(_, { G }, _)  :- G.
view_entry(C, (X, Y), E) :- view_entry(C, X, E), view_entry(C, Y, E).
view_entry(C, (X; Y), E) :- view_entry(C, X, E); view_entry(C, Y, E).
view_entry(C, E: V, _)   :- view_entry(C, V, E).

% Unifies `T` with the a compound term sharing the same functor as `P` and the
% same arity, but all arguments replaced with free variables. This "template"
% term may be used with `entry_tag` to search for matching tags, unifying their
% arguments with the arguments of `T`.
%
% Additionally unifies `Ps` with the arguments of `P` and `Ts` with the
% arguments of `T`. These lists may later be used with `pattern_arg` to post the
% final constraining goals of the argument patterns.
%
% Also works for non-compound terms, in which case `Ps` is unified with `[]`.
pattern_tag(P, Ps, T, Ts) :-
  (  compound(P)
  -> compound_name_arguments(P, F, Ps),
     same_length(Ps, Ts),
     compound_name_arguments(T, F, Ts)
  ;  P = T, Ps = []
  ).

pattern_arg(P, X) :-
  ( var(P) -> P = X
  ; P = ^P1 ->
    ( var(P1) -> nonvar(X)
    ; compound(P1) -> pattern_tag(P1, Ps, X, Xs), \+ maplist(pattern_arg, Ps, Xs)
    ; X \== P1
    )
  ; P = P1! -> ( var(P1) -> var(X) ; X == P1 )
  ; compound(P) -> pattern_tag(P, Ps, X, Xs), maplist(pattern_arg, Ps, Xs)
  ; P = X
  ).
