% Defines predicates available in the embedded Prolog runtime
% Required in order to build the initial Prolog state

% `@T` represents a tag `T`
:- op(50, fx, @).

% view_entry(Context: EntryTagContext, View, EntryId)
view_entry(C, { G }, E)  :- entry_tag(C, E, _), G.
view_entry(C, any, E)    :- entry_tag(C, E, _).
view_entry(C, @T, E)     :- entry_tag(C, E, T).
view_entry(C, (X, Y), E) :- view_entry(C, X, E), view_entry(C, Y, E).
view_entry(C, (X; Y), E) :- view_entry(C, X, E); view_entry(C, Y, E).
view_entry(C, E1: V, E)  :- view_entry(C, V, E1), entry_tag(C, E, _).
