use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub javap_parser, "/class_info/javap_parser.rs"); // synthesized by LALRPOP

#[test]
fn parse_java_util_ArrayList() {
    // Output from `javap -public -s java.util.ArrayList`
    const OUTPUT: &str = r##"
Compiled from "ArrayList.java"
public class java.util.ArrayList<E> extends java.util.AbstractList<E> implements java.util.List<E>, java.util.RandomAccess, java.lang.Cloneable, java.io.Serializable {

  public java.util.ArrayList(int);
    descriptor: (I)V

  public java.util.ArrayList();
    descriptor: ()V

  public java.util.ArrayList(java.util.Collection<? extends E>);
    descriptor: (Ljava/util/Collection;)V

  public void trimToSize();
    descriptor: ()V

  public void ensureCapacity(int);
    descriptor: (I)V

  public int size();
    descriptor: ()I

  public boolean isEmpty();
    descriptor: ()Z

  public boolean contains(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public int indexOf(java.lang.Object);
    descriptor: (Ljava/lang/Object;)I

  public int lastIndexOf(java.lang.Object);
    descriptor: (Ljava/lang/Object;)I

  public java.lang.Object clone();
    descriptor: ()Ljava/lang/Object;

  public java.lang.Object[] toArray();
    descriptor: ()[Ljava/lang/Object;

  public <T> T[] toArray(T[]);
    descriptor: ([Ljava/lang/Object;)[Ljava/lang/Object;

  public E get(int);
    descriptor: (I)Ljava/lang/Object;

  public E set(int, E);
    descriptor: (ILjava/lang/Object;)Ljava/lang/Object;

  public boolean add(E);
    descriptor: (Ljava/lang/Object;)Z

  public void add(int, E);
    descriptor: (ILjava/lang/Object;)V

  public E remove(int);
    descriptor: (I)Ljava/lang/Object;

  public boolean equals(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public int hashCode();
    descriptor: ()I

  public boolean remove(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public void clear();
    descriptor: ()V

  public boolean addAll(java.util.Collection<? extends E>);
    descriptor: (Ljava/util/Collection;)Z

  public boolean addAll(int, java.util.Collection<? extends E>);
    descriptor: (ILjava/util/Collection;)Z

  public boolean removeAll(java.util.Collection<?>);
    descriptor: (Ljava/util/Collection;)Z

  public boolean retainAll(java.util.Collection<?>);
    descriptor: (Ljava/util/Collection;)Z

  public java.util.ListIterator<E> listIterator(int);
    descriptor: (I)Ljava/util/ListIterator;

  public java.util.ListIterator<E> listIterator();
    descriptor: ()Ljava/util/ListIterator;

  public java.util.Iterator<E> iterator();
    descriptor: ()Ljava/util/Iterator;

  public java.util.List<E> subList(int, int);
    descriptor: (II)Ljava/util/List;

  public void forEach(java.util.function.Consumer<? super E>);
    descriptor: (Ljava/util/function/Consumer;)V

  public java.util.Spliterator<E> spliterator();
    descriptor: ()Ljava/util/Spliterator;

  public boolean removeIf(java.util.function.Predicate<? super E>);
    descriptor: (Ljava/util/function/Predicate;)Z

  public void replaceAll(java.util.function.UnaryOperator<E>);
    descriptor: (Ljava/util/function/UnaryOperator;)V

  public void sort(java.util.Comparator<? super E>);
    descriptor: (Ljava/util/Comparator;)V
}
    "##;

    match javap_parser::ClassInfoParser::new().parse(OUTPUT) {
        Err(lalrpop_util::ParseError::UnrecognizedToken { token, expected }) => {
            panic!(
                "unexpected token `{}` at {:?}; expected `{:?}`",
                token.1,
                OUTPUT[token.0..].chars().take(100).collect::<String>(),
                expected,
            )
        }
        Err(e) => {
            panic!("{}", format!("{e:?}"))
        }

        Ok(v) => {
            expect_test::expect![[r#"
                ClassInfo {
                    flags: Flags {
                        privacy: Public,
                        is_final: false,
                        is_synchronized: false,
                        is_native: false,
                    },
                    name: Id {
                        data: "java.util.ArrayList",
                    },
                    generics: [
                        Id {
                            data: "E",
                        },
                    ],
                    extends: Some(
                        ClassRef {
                            name: Id {
                                data: "java.util.AbstractList",
                            },
                            generics: [
                                TypeParameter(
                                    Id {
                                        data: "E",
                                    },
                                ),
                            ],
                        },
                    ),
                    implements: [
                        ClassRef {
                            name: Id {
                                data: "java.util.List",
                            },
                            generics: [
                                TypeParameter(
                                    Id {
                                        data: "E",
                                    },
                                ),
                            ],
                        },
                        ClassRef {
                            name: Id {
                                data: "java.util.RandomAccess",
                            },
                            generics: [],
                        },
                        ClassRef {
                            name: Id {
                                data: "java.lang.Cloneable",
                            },
                            generics: [],
                        },
                        ClassRef {
                            name: Id {
                                data: "java.io.Serializable",
                            },
                            generics: [],
                        },
                    ],
                    constructors: [
                        Constructor {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            args: [
                                Scalar(
                                    Int,
                                ),
                            ],
                            throws: [],
                            descriptor: Descriptor {
                                string: "(I)V",
                            },
                        },
                        Constructor {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            args: [],
                            throws: [],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                        Constructor {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            args: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.Collection",
                                            },
                                            generics: [
                                                Extends(
                                                    TypeParameter(
                                                        Id {
                                                            data: "E",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/Collection;)V",
                            },
                        },
                    ],
                    fields: [],
                    methods: [
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "trimToSize",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "ensureCapacity",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(I)V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "size",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Scalar(
                                Int,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()I",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "isEmpty",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "contains",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "indexOf",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Int,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)I",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "lastIndexOf",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Int,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)I",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "clone",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.lang.Object",
                                        },
                                        generics: [],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Ljava/lang/Object;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "toArray",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Array(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()[Ljava/lang/Object;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "toArray",
                            },
                            generics: [
                                Id {
                                    data: "T",
                                },
                            ],
                            argument_tys: [
                                Ref(
                                    Array(
                                        TypeParameter(
                                            Id {
                                                data: "T",
                                            },
                                        ),
                                    ),
                                ),
                            ],
                            return_ty: Ref(
                                Array(
                                    TypeParameter(
                                        Id {
                                            data: "T",
                                        },
                                    ),
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "([Ljava/lang/Object;)[Ljava/lang/Object;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "get",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "E",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(I)Ljava/lang/Object;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "set",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                                Ref(
                                    TypeParameter(
                                        Id {
                                            data: "E",
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "E",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(ILjava/lang/Object;)Ljava/lang/Object;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "add",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    TypeParameter(
                                        Id {
                                            data: "E",
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "add",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                                Ref(
                                    TypeParameter(
                                        Id {
                                            data: "E",
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(ILjava/lang/Object;)V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "remove",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "E",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(I)Ljava/lang/Object;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "equals",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "hashCode",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Scalar(
                                Int,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()I",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "remove",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "clear",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "addAll",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.Collection",
                                            },
                                            generics: [
                                                Extends(
                                                    TypeParameter(
                                                        Id {
                                                            data: "E",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/Collection;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "addAll",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.Collection",
                                            },
                                            generics: [
                                                Extends(
                                                    TypeParameter(
                                                        Id {
                                                            data: "E",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(ILjava/util/Collection;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "removeAll",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.Collection",
                                            },
                                            generics: [
                                                Wildcard,
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/Collection;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "retainAll",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.Collection",
                                            },
                                            generics: [
                                                Wildcard,
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/Collection;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "listIterator",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                            ],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.util.ListIterator",
                                        },
                                        generics: [
                                            TypeParameter(
                                                Id {
                                                    data: "E",
                                                },
                                            ),
                                        ],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(I)Ljava/util/ListIterator;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "listIterator",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.util.ListIterator",
                                        },
                                        generics: [
                                            TypeParameter(
                                                Id {
                                                    data: "E",
                                                },
                                            ),
                                        ],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Ljava/util/ListIterator;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "iterator",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.util.Iterator",
                                        },
                                        generics: [
                                            TypeParameter(
                                                Id {
                                                    data: "E",
                                                },
                                            ),
                                        ],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Ljava/util/Iterator;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "subList",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Int,
                                ),
                                Scalar(
                                    Int,
                                ),
                            ],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.util.List",
                                        },
                                        generics: [
                                            TypeParameter(
                                                Id {
                                                    data: "E",
                                                },
                                            ),
                                        ],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(II)Ljava/util/List;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "forEach",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.function.Consumer",
                                            },
                                            generics: [
                                                Super(
                                                    TypeParameter(
                                                        Id {
                                                            data: "E",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/function/Consumer;)V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "spliterator",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.util.Spliterator",
                                        },
                                        generics: [
                                            TypeParameter(
                                                Id {
                                                    data: "E",
                                                },
                                            ),
                                        ],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Ljava/util/Spliterator;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "removeIf",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.function.Predicate",
                                            },
                                            generics: [
                                                Super(
                                                    TypeParameter(
                                                        Id {
                                                            data: "E",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/function/Predicate;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "replaceAll",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.function.UnaryOperator",
                                            },
                                            generics: [
                                                TypeParameter(
                                                    Id {
                                                        data: "E",
                                                    },
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/function/UnaryOperator;)V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "sort",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.util.Comparator",
                                            },
                                            generics: [
                                                Super(
                                                    TypeParameter(
                                                        Id {
                                                            data: "E",
                                                        },
                                                    ),
                                                ),
                                            ],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/util/Comparator;)V",
                            },
                        },
                    ],
                }
            "#]]
            .assert_debug_eq(&v);
        }
    };
}

#[test]
fn parse_java_lang_Object() {
    // Output from `javap -public -s java.util.ArrayList`
    const OUTPUT: &str = r##"
    Compiled from "Object.java"
public class java.lang.Object {
  public java.lang.Object();
    descriptor: ()V

  public final native java.lang.Class<?> getClass();
    descriptor: ()Ljava/lang/Class;

  public native int hashCode();
    descriptor: ()I

  public boolean equals(java.lang.Object);
    descriptor: (Ljava/lang/Object;)Z

  public java.lang.String toString();
    descriptor: ()Ljava/lang/String;

  public final native void notify();
    descriptor: ()V

  public final native void notifyAll();
    descriptor: ()V

  public final void wait() throws java.lang.InterruptedException;
    descriptor: ()V

  public final void wait(long) throws java.lang.InterruptedException;
    descriptor: (J)V

  public final void wait(long, int) throws java.lang.InterruptedException;
    descriptor: (JI)V
}
    "##;

    match javap_parser::ClassInfoParser::new().parse(OUTPUT) {
        Err(lalrpop_util::ParseError::UnrecognizedToken { token, expected }) => {
            panic!(
                "unexpected token `{}` at {:?}; expected `{:?}`",
                token.1,
                OUTPUT[token.0..].chars().take(100).collect::<String>(),
                expected,
            )
        }
        Err(e) => {
            panic!("{}", format!("{e:?}"))
        }

        Ok(v) => {
            expect_test::expect![[r#"
                ClassInfo {
                    flags: Flags {
                        privacy: Public,
                        is_final: false,
                        is_synchronized: false,
                        is_native: false,
                    },
                    name: Id {
                        data: "java.lang.Object",
                    },
                    generics: [],
                    extends: None,
                    implements: [],
                    constructors: [
                        Constructor {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            args: [],
                            throws: [],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                    ],
                    fields: [],
                    methods: [
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: true,
                                is_synchronized: false,
                                is_native: true,
                            },
                            name: Id {
                                data: "getClass",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.lang.Class",
                                        },
                                        generics: [
                                            Wildcard,
                                        ],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Ljava/lang/Class;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: true,
                            },
                            name: Id {
                                data: "hashCode",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Scalar(
                                Int,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()I",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "equals",
                            },
                            generics: [],
                            argument_tys: [
                                Ref(
                                    Class(
                                        ClassRef {
                                            name: Id {
                                                data: "java.lang.Object",
                                            },
                                            generics: [],
                                        },
                                    ),
                                ),
                            ],
                            return_ty: Scalar(
                                Boolean,
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "(Ljava/lang/Object;)Z",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: false,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "toString",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                Class(
                                    ClassRef {
                                        name: Id {
                                            data: "java.lang.String",
                                        },
                                        generics: [],
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()Ljava/lang/String;",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: true,
                                is_synchronized: false,
                                is_native: true,
                            },
                            name: Id {
                                data: "notify",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: true,
                                is_synchronized: false,
                                is_native: true,
                            },
                            name: Id {
                                data: "notifyAll",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: true,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "wait",
                            },
                            generics: [],
                            argument_tys: [],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [
                                ClassRef {
                                    name: Id {
                                        data: "java.lang.InterruptedException",
                                    },
                                    generics: [],
                                },
                            ],
                            descriptor: Descriptor {
                                string: "()V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: true,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "wait",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Long,
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [
                                ClassRef {
                                    name: Id {
                                        data: "java.lang.InterruptedException",
                                    },
                                    generics: [],
                                },
                            ],
                            descriptor: Descriptor {
                                string: "(J)V",
                            },
                        },
                        Method {
                            flags: Flags {
                                privacy: Public,
                                is_final: true,
                                is_synchronized: false,
                                is_native: false,
                            },
                            name: Id {
                                data: "wait",
                            },
                            generics: [],
                            argument_tys: [
                                Scalar(
                                    Long,
                                ),
                                Scalar(
                                    Int,
                                ),
                            ],
                            return_ty: Ref(
                                TypeParameter(
                                    Id {
                                        data: "void",
                                    },
                                ),
                            ),
                            throws: [
                                ClassRef {
                                    name: Id {
                                        data: "java.lang.InterruptedException",
                                    },
                                    generics: [],
                                },
                            ],
                            descriptor: Descriptor {
                                string: "(JI)V",
                            },
                        },
                    ],
                }
            "#]]
            .assert_debug_eq(&v);
        }
    };
}
